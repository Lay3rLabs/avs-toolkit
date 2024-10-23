use anyhow::Result;
use serde::{Deserialize, Serialize};

use layer_wasi::{Reactor, Request, WasiPollable};

#[derive(Serialize, Debug)]
pub struct OllamaGenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum OllamaGenerateResponse {
    Error(OllamaGenerateErrorResponse),
    Success(OllamaGenerateSuccessResponse),
}

#[derive(Deserialize, Debug)]
pub struct OllamaGenerateErrorResponse {
    pub error: String,
}

#[derive(Deserialize, Debug)]
pub struct OllamaGenerateSuccessResponse {
    pub model: String,
    pub response: String,
}

pub async fn get_ollama_response(
    reactor: &Reactor,
    prompt: String,
) -> Result<OllamaGenerateSuccessResponse, String> {
    let mut req = Request::post("http://host.docker.internal:11434/api/generate")?;
    req.json(&OllamaGenerateRequest {
        model: "llama3.2:1b".to_string(),
        prompt,
        stream: false,
    })?;

    let res = reactor.send(req).await?;

    let raw = String::from_utf8(res.body.clone());
    println!("ollama response: {:?}", raw);

    match res.status {
        200 => res
            .json::<OllamaGenerateResponse>()
            .map(|r| match r {
                OllamaGenerateResponse::Error(e) => Err(e.error),
                OllamaGenerateResponse::Success(s) => Ok(s),
            })
            .or_else(|e| Err(format!("response parsing error ({e}). body: {raw:?}")))?,
        status => Err(format!("unexpected status code: {status}")),
    }
}
