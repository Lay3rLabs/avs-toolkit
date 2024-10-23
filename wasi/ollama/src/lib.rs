#[allow(warnings)]
mod bindings;
#[allow(warnings)]
mod ollama;

use anyhow::anyhow;
use bindings::{Guest, Output, TaskQueueInput};
use layer_wasi::{block_on, Reactor};
use serde::{Deserialize, Serialize};

struct Component;

impl Guest for Component {
    fn run_task(request: TaskQueueInput) -> Output {
        let TaskInput { prompt } = serde_json::from_slice(&request.request)
            .map_err(|e| anyhow!("Could not deserialize input request from JSON: {}", e))
            .unwrap();

        block_on(|reactor| get_ollama_response(reactor, prompt))
    }
}

async fn get_ollama_response(reactor: Reactor, prompt: String) -> Result<Vec<u8>, String> {
    let res = ollama::get_ollama_response(&reactor, prompt).await;

    // serialize JSON response

    let output = res
        .map(|r| TaskOutput::Success(r.response))
        .unwrap_or_else(|e| TaskOutput::Error(e.to_string()));

    serde_json::to_vec(&output).map_err(|e| e.to_string())
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskInput {
    pub prompt: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TaskOutput {
    Success(String),
    Error(String),
}

bindings::export!(Component with_types_in bindings);
