use std::str::FromStr;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use layer_wasi::{Reactor, Request, WasiPollable};
use serde_json::json;

#[derive(Serialize, Debug)]
pub struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<OllamaChatMessage>,
    pub stream: bool,
    pub options: serde_json::Value,
    pub tools: Vec<serde_json::Value>,
}

#[derive(Serialize, Debug)]
pub enum OllamaChatMessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OllamaChatMessage {
    pub role: String,
    pub content: String,
    pub tool_calls: Option<Vec<serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OllamaChatMessageToolCall {
    pub function: OllamaChatMessageToolCallFunction,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OllamaChatMessageToolCallFunction {
    pub name: String,
    pub arguments: serde_json::Map<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum OllamaChatResponse {
    Error(OllamaChatErrorResponse),
    Success(OllamaChatSuccessResponse),
}

#[derive(Deserialize, Debug)]
pub struct OllamaChatErrorResponse {
    pub error: String,
}

#[derive(Deserialize, Debug)]
pub struct OllamaChatSuccessResponse {
    pub model: String,
    pub message: OllamaChatMessage,
    pub decision: Option<bool>,
}

pub async fn get_ollama_response(
    reactor: &Reactor,
    prompt: String,
) -> Result<OllamaChatSuccessResponse, String> {
    let mut req = Request::post("http://host.docker.internal:11434/api/chat")?;
    req.json(&OllamaChatRequest {
        model: "llama3.2:1b".to_string(),
        messages: vec![
            OllamaChatMessage {
                role: "system".to_string(),
                content: "You are a bouncer, deciding who is allowed to join an organization based purely on good vibes. Engage in a conversation with the user, and make a decision only once you are confident you have enough information.".to_string(),
                tool_calls: None,
            },
            OllamaChatMessage {
                role: "user".to_string(),
                content: prompt,
                tool_calls: None,
            }
        ],
        stream: false,
        options: json!({
            "seed": 42,
            "temperature": 0.7
        }),
        tools: vec![json!({
            "type": "function",
            "function": {
                "name": "make_decision",
                "description": "Make a decision about whether or not to allow the user into the organization.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "decision": {
                            "type": "boolean",
                            "description": "Whether or not to allow the user into the organization."
                        }
                    },
                    "required": ["decision"]
                }
            }
        })],
    })?;

    let res = reactor.send(req).await?;

    let raw = String::from_utf8(res.body.clone());
    println!("ollama response: {:?}", raw);

    match res.status {
        200 => res
            .json::<OllamaChatResponse>()
            .map(|r| match r {
                OllamaChatResponse::Error(e) => Err(e.error),
                OllamaChatResponse::Success(s) => Ok(s),
            })
            .or_else(|e| Err(format!("response parsing error ({e}). body: {raw:?}")))?,
        status => Err(format!("unexpected status code: {status}")),
    }
}
