use anyhow::{bail, Result};
use reqwest::Client;
use serde_json::json;
use std::path::PathBuf;
use tokio::fs;

pub async fn deploy(
    address: String,
    name: String,
    digest: String,
    wasm_source: String,
    trigger: String,
    permissions_json: String,
    envs_json: String,
) -> Result<()> {
    let client = Client::new();

    // Prepare the JSON body
    let body = json!({
        "name": name,
        "digest": digest,
        "trigger": {
            "cron": {
                "schedule": trigger
            }
        },
        "permissions": serde_json::from_str::<serde_json::Value>(&permissions_json).unwrap(),
        "envs": serde_json::from_str::<Vec<Vec<String>>>(&envs_json).unwrap(),
    });

    // Check if wasm_source is a URL or a local file path
    if wasm_source.starts_with("http://") || wasm_source.starts_with("https://") {
        // wasm_source is a URL, include wasmUrl in the body
        let mut json_body = body.clone();
        json_body["wasmUrl"] = json!(wasm_source);

        // Send the request with wasmUrl in JSON
        let response = client
            .post(&format!("{}/app", address))
            .json(&json_body)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if response.status().is_success() {
            println!("Deployment successful!");
        } else {
            bail!("Error: {:?}", response.text().await?);
        }
    } else {
        // wasm_source is a local file, read the binary
        let wasm_binary = fs::read(wasm_source).await?;

        // Send the request with the binary file and JSON body
        let response = client
            .post(&format!("{}/app", address))
            .header("Content-Type", "application/json") // Content-Type remains application/json
            .body(wasm_binary) // Binary data goes here
            .json(&body) // JSON body goes here
            .send()
            .await?;

        if response.status().is_success() {
            println!("Deployment successful!");
        } else {
            bail!("Error: {:?}", response.text().await?);
        }
    }

    Ok(())
}
