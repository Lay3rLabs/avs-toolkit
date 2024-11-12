use anyhow::Result;
use serde::{Deserialize, Serialize};

use layer_wasi::{Reactor, Request, WasiPollable};

use crate::{
    session::{Session, SessionMessage},
    Env, TaskInput,
};

use super::Provider;

#[derive(Serialize, Debug)]
pub struct GroqChatRequest {
    pub model: String,
    pub messages: Vec<GroqChatMessage>,
    pub stream: bool,
    pub seed: Option<u64>,
    pub temperature: Option<f32>,
    pub tools: Vec<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct GroqChatChoice {
    pub message: GroqChatMessage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroqChatMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<GroqChatMessageToolCall>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroqChatMessageToolCall {
    pub function: GroqChatMessageToolCallFunction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroqChatMessageToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum GroqChatResponse {
    Error(GroqChatErrorResponse),
    Success(GroqChatSuccessResponse),
}

#[derive(Deserialize, Debug)]
pub struct GroqChatErrorResponse {
    pub error: String,
}

#[derive(Deserialize, Debug)]
pub struct GroqChatSuccessResponse {
    pub choices: Vec<GroqChatChoice>,
}

impl From<&SessionMessage> for GroqChatMessage {
    fn from(value: &SessionMessage) -> Self {
        Self {
            role: value.role.clone(),
            content: Some(value.content.clone()),
            tool_calls: value.decision.map(|d| {
                vec![GroqChatMessageToolCall {
                    function: GroqChatMessageToolCallFunction {
                        name: "make_decision".to_string(),
                        arguments: format!("{{\"decision\": {d}}}"),
                    },
                }]
            }),
        }
    }
}

pub struct GroqProvider {
    api_key: String,
}

impl Provider for GroqProvider {
    const NAME: &'static str = "groq";

    fn new() -> Result<Self, String> {
        let api_key = std::env::var("GROQ_API_KEY")
            .map_err(|e| format!("missing env var `GROQ_API_KEY`: {e}"))?;

        Ok(Self { api_key })
    }

    async fn process(
        &self,
        reactor: &Reactor,
        env: &Env,
        input: &TaskInput,
    ) -> Result<Session, String> {
        let mut session = Session::load(
            reactor,
            env,
            &input.dao,
            &input.address,
            "llama3-groq-70b-8192-tool-use-preview",
        )
        .await?;

        session.validate_message_id(input.message_id)?;
        session.add_user_message(&input.message);

        let mut req = Request::post("https://api.groq.com/openai/v1/chat/completions")?;
        req.headers.push((
            "Authorization".to_string(),
            format!("Bearer {}", self.api_key),
        ));
        req.json(&GroqChatRequest {
            model: session.model.clone(),
            messages: session.messages.iter().map(|m| m.into()).collect(),
            seed: Some(session.seed),
            temperature: Some(session.temperature),
            tools: session.tools.clone(),
            stream: false,
        })?;

        let res = reactor.send(req).await?;

        let raw = String::from_utf8(res.body.clone());
        dbg!("groq response: {:#?}", &raw);

        if res.status != 200 {
            let body_text = String::from_utf8(res.body.clone()).unwrap();
            return Err(format!(
                "unexpected status code: {} body: {}",
                res.status, body_text
            ));
        }

        match res.json::<GroqChatResponse>() {
            Ok(response) => match response {
                GroqChatResponse::Error(e) => Err(e.error),
                GroqChatResponse::Success(response) => {
                    if response.choices.is_empty() {
                        return Err("expected at least one choice".to_string());
                    }

                    session.add_ai_message(
                        &response.choices[0]
                            .message
                            .content
                            .clone()
                            .unwrap_or_default(),
                        response.choices[0]
                            .message
                            .tool_calls
                            .as_ref()
                            .map(|calls| {
                                calls.len() == 1
                                    && serde_json::from_str::<serde_json::Value>(
                                        &calls[0].function.arguments,
                                    )
                                    .unwrap()["decision"]
                                        .as_bool()
                                        .unwrap()
                            }),
                    );

                    Ok(session)
                }
            },
            Err(e) => Err(format!("response parsing error ({e}). body: {raw:?}")),
        }
    }
}
