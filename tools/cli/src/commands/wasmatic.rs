use super::wasmatic_cron_bindings as cron_bindings;
use super::wasmatic_task_bindings as task_bindings;
use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::fs;
use wasmtime::{
    component::{Component, Linker},
    Config, Engine,
};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

use crate::context::AppContext;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Trigger {
    #[serde(rename_all = "camelCase")]
    Cron { schedule: String },
    #[serde(rename_all = "camelCase")]
    Queue {
        task_queue_addr: String,
        hd_index: u32,
        poll_interval: u32,
    },
}

#[allow(clippy::too_many_arguments)]
pub async fn deploy(
    ctx: &AppContext,
    name: String,
    digest: Option<String>,
    wasm_source: String,
    trigger: Trigger,
    permissions_json: String,
    env_pairs: Vec<String>,
    testable: bool,
) -> Result<()> {
    let endpoints = &ctx.chain_info()?.wasmatic.endpoints;
    let client = Client::new();

    let envs = env_pairs
        .iter()
        .map(|env| {
            let (k, v) = env.split_once('=').unwrap();
            vec![k.to_string(), v.to_string()]
        })
        .collect::<Vec<Vec<String>>>();

    // Prepare the JSON body
    let body = json!({
        "name": name,
        "trigger": trigger,
        "permissions": serde_json::from_str::<serde_json::Value>(&permissions_json).unwrap(),
        "envs": envs,
        "testable": testable,
    });

    // Check if wasm_source is a URL or a local file path
    let json_body = if wasm_source.starts_with("http://") || wasm_source.starts_with("https://") {
        // wasm_source is a URL, include wasmUrl in the body
        let mut json_body = body.clone();

        if digest.is_none() {
            bail!("Error: You need to provide sha256 sum digest if wasm source is an url")
        }

        json_body["digest"] = json!(digest.unwrap());
        json_body["wasmUrl"] = json!(wasm_source);

        json_body
    } else {
        let mut json_body = body.clone();

        // wasm_source is a local file, read the binary
        let wasm_binary = fs::read(wasm_source).await?;

        // calculate sha256sum
        let mut hasher = Sha256::new();
        hasher.update(&wasm_binary);
        let result = hasher.finalize();
        json_body["digest"] = json!(format!("sha256:{:x}", result));

        futures::future::join_all(endpoints.iter().map(|endpoint| {
            let client = client.clone();
            let wasm_binary = wasm_binary.clone();
            async move {
                let response = client
                    .post(format!("{}/upload", endpoint))
                    .body(wasm_binary) // Binary data goes here
                    .send()
                    .await?;
                if !response.status().is_success() {
                    bail!("Error: {:?}", response.text().await?);
                }
                Ok(())
            }
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<()>, _>>()?;

        json_body
    };

    futures::future::join_all(endpoints.iter().map(|endpoint| {
        let client = client.clone();
        let json_body = json_body.clone();
        async move {
            // Send the request with wasmUrl in JSON
            let response = client
                .post(format!("{}/app", endpoint))
                .json(&json_body)
                .send()
                .await?;

            if response.status().is_success() {
                println!("Deployment successful to operator: {endpoint}");
            } else {
                bail!("Error: {:?}", response.text().await?);
            }
            Ok(())
        }
    }))
    .await
    .into_iter()
    .collect::<Result<Vec<()>, _>>()?;

    Ok(())
}

pub async fn remove(ctx: &AppContext, app_name: String) -> Result<()> {
    let endpoints = &ctx.chain_info()?.wasmatic.endpoints;
    let client = Client::new();

    // Prepare the JSON body
    let body = json!({
        "apps": [app_name],
    });

    futures::future::join_all(endpoints.iter().map(|endpoint| {
        let client = client.clone();
        let body = body.clone();
        async move {
            // Send the DELETE request
            let response = client
                .delete(format!("{}/app", endpoint))
                .json(&body) // JSON body goes here
                .send()
                .await?;

            // Check if the request was successful
            if !response.status().is_success() {
                bail!("Error: {:?}", response.text().await?);
            }
            Ok(())
        }
    }))
    .await
    .into_iter()
    .collect::<Result<Vec<()>, _>>()?;

    Ok(())
}

/// This is the return value for error (message) or success (output) cases, if needed later
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TestOutput {
    pub message: Option<String>,
    pub output: Option<Value>,
}

pub async fn test(ctx: &AppContext, app_name: String, input: Option<String>) -> Result<()> {
    let endpoints = &ctx.chain_info()?.wasmatic.endpoints;
    let client = Client::new();

    // Prepare the JSON body
    let body = if let Some(input) = input {
        json!({
            "name": app_name,
            "input": serde_json::from_str::<Value>(&input)?,
        })
    } else {
        json!({
            "name": app_name,
        })
    };

    futures::future::join_all(endpoints.iter().map(|endpoint| {
        let client = client.clone();
        let body = body.clone();
        async move {
            // Send the POST request
            let response = client
                .post(format!("{}/test", endpoint))
                .header("Content-Type", "application/json")
                .json(&body) // Send the JSON body
                .send()
                .await?;

            // Check if the request was successful
            if response.status().is_success() {
                println!("Test executed successfully!");
                let response_text = response.text().await?;
                println!("Output for operator `{endpoint}`: {}", response_text);
            } else {
                // let json: TestOutput = response.json().await?;
                let json = response.text().await?;
                bail!("{}", json);
            }
            Ok(())
        }
    }))
    .await
    .into_iter()
    .collect::<Result<Vec<()>, _>>()?;

    Ok(())
}

pub async fn run(
    wasm_source: String,
    cron_trigger: bool,
    env_pairs: Vec<String>,
    app_cache_path: PathBuf,
    input: Option<String>,
) -> Result<String> {
    // Check if wasm_source is a URL or a local file path
    let wasm_binary = if wasm_source.starts_with("http://") || wasm_source.starts_with("https://") {
        match reqwest::get(wasm_source).await {
            Ok(res) if res.status().is_success() => match res.bytes().await {
                Ok(bytes) => bytes.to_vec(),
                Err(err) => Err(err).context("Failed to download from specified URL")?,
            },
            Ok(res) => bail!(
                "Failed to download from specified URL: {status}",
                status = res.status()
            ),
            Err(err) => Err(err).context("Failed to download from specified URL")?,
        }
    } else {
        // wasm_source is a local file, read the binary
        fs::read(wasm_source).await?
    };

    let trigger = if cron_trigger {
        TriggerRequest::Cron
    } else {
        TriggerRequest::Queue(input.unwrap_or_default().into_bytes())
    };

    let envs = env_pairs
        .iter()
        .map(|env| {
            env.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .unwrap()
        })
        .collect::<Vec<(String, String)>>();

    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    let engine = Engine::new(&config).unwrap();

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_async(&mut linker).unwrap();
    wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker).unwrap();

    if !app_cache_path.is_dir() {
        tokio::fs::create_dir(&app_cache_path).await?;
    }

    let component = Component::new(&engine, wasm_binary)?;

    let output = instantiate_and_invoke(
        &envs,
        &app_cache_path,
        &engine,
        &linker,
        &component,
        trigger,
    )
    .await
    .map_err(|msg| anyhow::anyhow!("{}", msg))?;

    Ok(std::string::String::from_utf8(output).expect("Output is invalid utf8"))
}

enum TriggerRequest {
    Cron,
    Queue(Vec<u8>),
}

async fn instantiate_and_invoke(
    envs: &[(String, String)],
    app_cache_path: &PathBuf,
    engine: &Engine,
    linker: &Linker<Host>,
    component: &Component,
    trigger: TriggerRequest,
) -> Result<Vec<u8>, String> {
    let mut builder = WasiCtxBuilder::new();
    if !envs.is_empty() {
        builder.envs(envs);
    }
    builder
        .preopened_dir(app_cache_path, ".", DirPerms::all(), FilePerms::all())
        .expect("preopen failed");
    let ctx = builder.build();

    let host = Host {
        table: wasmtime::component::ResourceTable::new(),
        ctx,
        http: WasiHttpCtx::new(),
    };
    let mut store = wasmtime::Store::new(engine, host);
    match trigger {
        TriggerRequest::Cron => {
            if component
                .component_type()
                .get_export(engine, "run-cron")
                .is_none()
            {
                return Err("Wasm component is missing the expected function export `run-cron` for CRON trigger app".to_string());
            }
            let bindings = cron_bindings::CronJob::instantiate_async(&mut store, component, linker)
                .await
                .expect("Wasm instantiate failed");

            bindings
                .call_run_cron(&mut store)
                .await
                .expect("Failed to call invoke cron job")
        }
        TriggerRequest::Queue(request) => {
            if component
                .component_type()
                .get_export(engine, "run-task")
                .is_none()
            {
                return Err("Wasm component is missing the expected function export `run-task` for task queue trigger app".to_string());
            }
            let bindings =
                task_bindings::TaskQueue::instantiate_async(&mut store, component, linker)
                    .await
                    .expect("Wasm instantiate failed");

            let input = task_bindings::lay3r::avs::types::TaskQueueInput {
                timestamp: get_time(),
                request,
            };
            bindings
                .call_run_task(&mut store, &input)
                .await
                .expect("Failed to run task")
        }
    }
}

fn get_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

struct Host {
    pub(crate) table: wasmtime::component::ResourceTable,
    pub(crate) ctx: WasiCtx,
    pub(crate) http: WasiHttpCtx,
}

impl WasiView for Host {
    fn table(&mut self) -> &mut wasmtime_wasi::ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

impl WasiHttpView for Host {
    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }
}
