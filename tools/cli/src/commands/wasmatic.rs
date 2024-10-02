use anyhow::{bail, Result};
use reqwest::Client;
use serde_json::json;
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
        let response = client
            .post(&format!("{}/upload", address))
            .body(wasm_binary) // Binary data goes here
            .send()
            .await?;
        if !response.status().is_success() {
            bail!("Error: {:?}", response.text().await?);
        }

        // Send the request with the binary file and JSON body
        let response = client
            .post(&format!("{}/app", address))
            .header("Content-Type", "application/json") // Content-Type remains application/json
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

pub async fn remove(address: String, app_name: String) -> Result<()> {
    let client = Client::new();

    // Prepare the JSON body
    let body = json!({
        "apps": [app_name],
    });

    // Send the DELETE request
    let response = client
        .delete(&format!("{}/app", address))
        .header("Content-Type", "application/json")
        .json(&body) // JSON body goes here
        .send()
        .await?;

    // Check if the request was successful
    if !response.status().is_success() {
        bail!("Error: {:?}", response.text().await?);
    }

    Ok(())
}

pub async fn test(
    address: String,
    app_name: String,
    input: serde_json::Value, // This allows for any valid JSON input
) -> Result<()> {
    let client = Client::new();

    // Prepare the JSON body
    let body = json!({
        "name": app_name,
        "input": input,
    });

    // Send the POST request
    let response = client
        .post(&format!("{}/test", address))
        .header("Content-Type", "application/json")
        .json(&body) // Send the JSON body
        .send()
        .await?;

    // Check if the request was successful
    if response.status().is_success() {
        println!("Test executed successfully!");
        let response_text = response.text().await?;
        println!("Response: {}", response_text);
    } else {
        bail!("Error: {:?}", response.text().await?);
    }

    Ok(())
}
