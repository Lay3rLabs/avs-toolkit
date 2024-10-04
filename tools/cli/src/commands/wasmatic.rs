use anyhow::{bail, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::fs;

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

pub async fn test(ctx: &AppContext, app_name: String, input: String) -> Result<()> {
    let endpoints = &ctx.chain_info()?.wasmatic.endpoints;
    let client = Client::new();

    // Parse input into json
    let input: Value = serde_json::from_str(&input)?;

    // Prepare the JSON body
    let body = json!({
        "name": app_name,
        "input": input,
    });

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
