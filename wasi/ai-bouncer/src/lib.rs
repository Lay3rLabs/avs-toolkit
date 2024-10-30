#[allow(warnings)]
mod bindings;
#[allow(warnings)]
mod ollama;

use bindings::{Guest, Output, TaskQueueInput};
use layer_wasi::{block_on, Reactor};
use serde::{Deserialize, Serialize};

struct Component;

impl Guest for Component {
    fn run_task(request: TaskQueueInput) -> Output {
        // lock so only one can run at a time
        let lock = std::path::Path::new("./lock");
        match std::fs::OpenOptions::new().create_new(true).open(lock) {
            Ok(_) => {}
            Err(_) => return Err("another instance is running".to_string()),
        }

        let res = match serde_json::from_slice(&request.request) {
            Ok(input) => block_on(|reactor| get_output(reactor, input)),
            Err(e) => serde_json::to_vec(&TaskOutput::Error(format!(
                "Could not deserialize input request from JSON: {}",
                e
            )))
            .map_err(|e| e.to_string()),
        };

        // remove lock
        let _ = std::fs::remove_file(lock);

        res
    }
}

async fn get_output(reactor: Reactor, input: TaskInput) -> Result<Vec<u8>, String> {
    let output = ollama::get_output(&reactor, &input)
        .await
        .map(TaskOutput::Success)
        .unwrap_or_else(TaskOutput::Error);
    serde_json::to_vec(&output).map_err(|e| e.to_string())
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskInput {
    /// the session ID of the address being evaluated
    pub session_id: String,
    /// the address being evaluated. only needed on first message (where ID = 0)
    pub address: Option<String>,
    /// the incrementing message index, starting at 0
    pub message_id: u16,
    /// the next message in the conversation
    pub message: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TaskOutput {
    Success(TaskOutputSuccess),
    Error(String),
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskOutputSuccess {
    /// the session ID of the address being evaluated
    pub session_id: String,
    /// the address being evaluated
    pub address: String,
    /// the message ID being responded to
    pub message_id: u16,
    /// the response to the message
    pub response: String,
    /// the decision made by the AI bouncer, which will be present once
    /// finalized
    pub decision: Option<bool>,
}

bindings::export!(Component with_types_in bindings);
