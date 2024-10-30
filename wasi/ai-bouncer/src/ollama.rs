use std::str::FromStr;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use layer_wasi::{Reactor, Request, WasiPollable};
use serde_json::json;

use crate::{TaskInput, TaskOutput, TaskOutputSuccess};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct AiBouncerSession {
    pub session_id: String,
    pub address: String,
    pub decision: Option<bool>,
    pub model: String,
    pub messages: Vec<OllamaChatMessage>,
    pub options: serde_json::Value,
    pub tools: Vec<serde_json::Value>,
}

pub async fn get_output(reactor: &Reactor, input: &TaskInput) -> Result<TaskOutputSuccess, String> {
    // read from file
    let path = format!("./sessions/{}.json", input.session_id);
    let path = std::path::Path::new(&path);

    let mut session: AiBouncerSession;
    if std::path::Path::exists(path) {
        if let Ok(content) = std::fs::read_to_string(path) {
            session = serde_json::from_str(&content).unwrap();
        } else {
            return Err("failed to read session file".to_string());
        }
    } else {
        std::fs::create_dir_all("./sessions")
            .map_err(|e| format!("failed to create sessions directory: {e}"))?;

        let address = input
            .address
            .as_ref()
            .ok_or("address is required on first message")?;

        session = AiBouncerSession {
            session_id: input.session_id.clone(),
            address: address.to_string(),
            decision: None,
            model: "llama3-groq-tool-use".to_string(),
            messages: vec![
                OllamaChatMessage {
                    role: "system".to_string(),
                    content: "You are a bouncer, deciding who is allowed to join an organization based purely on good vibes. Engage in a conversation with the user, and make a decision using the make_decision tool only once you are confident you have enough information. You should not make a decision until exchanging at least a few messages back and forth.".to_string(),
                    tool_calls: None,
                },
            ],
            options: json!({
                "seed": 42,
                "temperature": 0.7
            }),
            // tools: vec![],
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
        };
    }

    // ensure the session does not yet have a decision
    if session.decision.is_some() {
        return Err("a decision has already been made".to_string());
    }

    let user_messages = session
        .messages
        .iter()
        .filter(|m| m.role == "user")
        .collect::<Vec<_>>();
    // ensure this is the next user message
    if input.message_id != user_messages.len() as u16 {
        return Err(format!(
            "message_id mismatch: expected {} but got {}",
            user_messages.len(),
            input.message_id
        ));
    }

    session.messages.push(OllamaChatMessage {
        role: "user".to_string(),
        content: input.message.clone(),
        tool_calls: None,
    });

    let mut req = Request::post("http://host.docker.internal:11434/api/chat")?;
    req.json(&OllamaChatRequest {
        model: session.model.clone(),
        messages: session.messages.clone(),
        options: session.options.clone(),
        tools: session.tools.clone(),
        stream: false,
    })?;

    let res = reactor.send(req).await?;

    let raw = String::from_utf8(res.body.clone());
    dbg!("ollama response: {:#?}", &raw);

    if res.status != 200 {
        return Err(format!("unexpected status code: {}", res.status));
    }

    match res.json::<OllamaChatResponse>() {
        Ok(response) => match response {
            OllamaChatResponse::Error(e) => Err(e.error),
            OllamaChatResponse::Success(mut s) => {
                session.messages.push(s.message.clone());
                if let Some(calls) = &s.message.tool_calls {
                    if calls.len() == 1
                        && calls[0].is_object()
                        && calls[0]["function"].is_object()
                        && calls[0]["function"]["name"].as_str() == Some("make_decision")
                        && calls[0]["function"]["arguments"].is_object()
                        && calls[0]["function"]["arguments"]["decision"].is_boolean()
                    {
                        let decision = calls[0]["function"]["arguments"]["decision"]
                            .as_bool()
                            .unwrap();

                        session.decision = Some(decision);
                        s.decision = Some(decision);
                    }
                }

                std::fs::write(path, serde_json::to_string(&session).unwrap())
                    .map_err(|e| format!("failed to write session file: {e}"))?;

                Ok(TaskOutputSuccess {
                    session_id: session.session_id,
                    message_id: input.message_id,
                    address: session.address,
                    response: s.message.content,
                    decision: s.decision,
                })
            }
        },
        Err(e) => Err(format!("response parsing error ({e}). body: {raw:?}")),
    }
}
