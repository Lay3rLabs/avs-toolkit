mod cron_bindings;
mod task_bindings;
use anyhow::{bail, Context, Result};
use avs_toolkit_shared::file::WasmFile;
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

pub async fn wasm_arg_to_file(wasm_arg: String) -> Result<WasmFile> {
    // Check if wasm_source is a URL or a local file path
    if wasm_arg.starts_with("http://") || wasm_arg.starts_with("https://") {
        Ok(WasmFile::Url(wasm_arg))
    } else {
        fs::read(wasm_arg)
            .await
            .map_err(|e| e.into())
            .map(WasmFile::Bytes)
    }
}

pub async fn run(
    wasm_file: WasmFile,
    cron_trigger: bool,
    env_pairs: Vec<String>,
    app_cache_path: PathBuf,
    input: Option<String>,
) -> Result<String> {
    // Check if wasm_source is a URL or a local file path
    let wasm_binary = match wasm_file {
        WasmFile::Url(url) => match reqwest::get(url).await {
            Ok(res) if res.status().is_success() => match res.bytes().await {
                Ok(bytes) => bytes.to_vec(),
                Err(err) => Err(err).context("Failed to download from specified URL")?,
            },
            Ok(res) => bail!(
                "Failed to download from specified URL: {status}",
                status = res.status()
            ),
            Err(err) => Err(err).context("Failed to download from specified URL")?,
        },
        WasmFile::Bytes(bytes) => bytes,
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
    builder.inherit_stdio();
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
