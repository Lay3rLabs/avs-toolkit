use std::sync::Arc;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::file::WasmFile;

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    client: reqwest::Client,
    endpoints: Vec<String>,
    name: String,
    digest: Option<String>,
    wasm_file: WasmFile,
    trigger: Trigger,
    permissions_json: String,
    env_pairs: Vec<String>,
    testable: bool,
    on_deploy_success: impl Fn(&str),
) -> Result<()> {
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
    let json_body = match wasm_file {
        WasmFile::Url(wasm_url) => {
            // wasm_source is a URL, include wasmUrl in the body
            let mut json_body = body.clone();

            if digest.is_none() {
                bail!("Error: You need to provide sha256 sum digest if wasm source is an url")
            }

            json_body["digest"] = json!(digest.unwrap());
            json_body["wasmUrl"] = json!(wasm_url);

            json_body
        }
        WasmFile::Bytes(wasm_binary) => {
            let mut json_body = body.clone();

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
        }
    };

    let on_deploy_success = Arc::new(on_deploy_success);

    futures::future::join_all(endpoints.iter().map(|endpoint| {
        let client = client.clone();
        let json_body = json_body.clone();
        let on_deploy_success = on_deploy_success.clone();
        async move {
            // Send the request with wasmUrl in JSON
            let response = client
                .post(format!("{}/app", endpoint))
                .json(&json_body)
                .send()
                .await?;

            if response.status().is_success() {
                on_deploy_success(endpoint);
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

pub async fn remove(
    client: reqwest::Client,
    endpoints: Vec<String>,
    app_name: String,
    on_remove_success: impl Fn(&str),
) -> Result<()> {
    // Prepare the JSON body
    let body = json!({
        "apps": [app_name],
    });

    let on_remove_success = Arc::new(on_remove_success);

    futures::future::join_all(endpoints.iter().map(|endpoint| {
        let client = client.clone();
        let body = body.clone();
        let on_remove_success = on_remove_success.clone();
        async move {
            // Send the DELETE request
            let response = client
                .delete(format!("{}/app", endpoint))
                .json(&body) // JSON body goes here
                .send()
                .await?;

            // Check if the request was successful
            if response.status().is_success() {
                on_remove_success(endpoint);
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

#[derive(Deserialize, Debug, Serialize)]
pub struct InfoResponse {
    pub endpoint: String,
    pub response: EndpointInfoResponse,
}
#[derive(Deserialize, Debug, Serialize)]
pub struct EndpointInfoResponse {
    pub operators: Vec<String>,
}

pub async fn info(
    client: reqwest::Client,
    endpoints: Vec<String>,
    on_info_success: impl Fn(&InfoResponse),
) -> Result<Vec<InfoResponse>> {
    let on_info_success = Arc::new(on_info_success);

    futures::future::join_all(endpoints.into_iter().map(|endpoint| {
        let client = client.clone();
        let on_info_success = on_info_success.clone();
        async move {
            let response = client.get(format!("{}/info", endpoint)).send().await?;

            if response.status().is_success() {
                let response: EndpointInfoResponse = response.json().await?;

                let result = InfoResponse { endpoint, response };

                on_info_success(&result);

                Ok(result)
            } else {
                bail!("Error: {:?}", response.text().await?);
            }
        }
    }))
    .await
    .into_iter()
    .collect::<Result<Vec<InfoResponse>>>()
}

// Define the structure to deserialize the response
#[derive(Deserialize, Debug, Serialize)]
pub struct AppResponse {
    apps: Vec<AppInfo>,
    digests: Vec<String>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct AppInfo {
    name: String,
    digest: String,
    trigger: Trigger,
    permissions: Value,
    testable: bool,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Queue {
    task_queue_addr: String,
    hd_index: u32,
    poll_interval: u32,
}

pub async fn app(client: reqwest::Client, endpoint: String) -> Result<AppResponse> {
    let response = client.get(format!("{}/app", endpoint)).send().await?;

    if !response.status().is_success() {
        bail!("Error: {:?}", response.text().await?);
    }

    response.json().await.map_err(|e| e.into())
}

/// This is the return value for error (message) or success (output) cases, if needed later
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TestOutput {
    pub message: Option<String>,
    pub output: Option<Value>,
}

pub struct TestResult {
    pub endpoint: String,
    pub response_text: String,
}

pub async fn test(
    client: reqwest::Client,
    endpoints: Vec<String>,
    app_name: String,
    input: Option<String>,
    on_test_result: impl Fn(&TestResult),
) -> Result<Vec<TestResult>> {
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

    let on_test_result = Arc::new(on_test_result);

    let results = futures::future::join_all(endpoints.into_iter().map(|endpoint| {
        let client = client.clone();
        let body = body.clone();
        let on_test_result = on_test_result.clone();
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
                let response_text = response.text().await?;

                let result = TestResult {
                    endpoint,
                    response_text,
                };

                on_test_result(&result);

                Ok(result)
            } else {
                // let json: TestOutput = response.json().await?;
                let json = response.text().await?;
                bail!("{}", json);
            }
        }
    }))
    .await
    .into_iter()
    .collect::<Result<Vec<TestResult>>>()?;

    Ok(results)
}