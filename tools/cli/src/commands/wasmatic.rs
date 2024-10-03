use anyhow::{bail, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::fs;

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

pub async fn deploy(
    address: String,
    name: String,
    digest: Option<String>,
    wasm_source: String,
    trigger: Trigger,
    permissions_json: String,
    env_pairs: Vec<String>,
    testable: bool,
) -> Result<()> {
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

        let response = client
            .post(format!("{}/upload", address))
            .body(wasm_binary) // Binary data goes here
            .send()
            .await?;
        if !response.status().is_success() {
            bail!("Error: {:?}", response.text().await?);
        }

        json_body
    };

    // Send the request with wasmUrl in JSON
    let response = client
        .post(format!("{}/app", address))
        .json(&json_body)
        .header("Content-Type", "application/json")
        .send()
        .await?;

    if response.status().is_success() {
        println!("Deployment successful!");
    } else {
        bail!("Error: {:?}", response.text().await?);
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
        .delete(format!("{}/app", address))
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

/// This is the return value for error (message) or success (output) cases, if needed later
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TestOutput {
    pub message: Option<String>,
    pub output: Option<Value>,
}

pub async fn test(address: String, app_name: String, input: String) -> Result<()> {
    let client = Client::new();

    // Parse input into json
    let input: Value = serde_json::from_str(&input)?;

    // Prepare the JSON body
    let body = json!({
        "name": app_name,
        "input": input,
    });

    // Send the POST request
    let response = client
        .post(format!("{}/test", address))
        .header("Content-Type", "application/json")
        .json(&body) // Send the JSON body
        .send()
        .await?;

    // Check if the request was successful
    if response.status().is_success() {
        println!("Test executed successfully!");
        let response_text = response.text().await?;
        println!("Output: {}", response_text);
    } else {
        // let json: TestOutput = response.json().await?;
        let json = response.text().await?;
        bail!("{}", json);
    }

    Ok(())
}
