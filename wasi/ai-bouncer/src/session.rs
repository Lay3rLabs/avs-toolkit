use anyhow::Result;
use base64::{prelude::BASE64_STANDARD, Engine};
use layer_wasi::{Reactor, Request, WasiPollable};
use serde::{Deserialize, Serialize};

use serde_json::json;

use crate::Env;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionMessage {
    /// the role of the sender of this message.
    pub role: String,
    /// the content of the message.
    pub content: String,
    /// the decision made by this message. this must be from the AI.
    pub decision: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    /// the DAO address.
    pub dao: String,
    /// the address of the user interacting with this session.
    pub address: String,
    /// the decision made by the AI in this session.
    pub decision: Option<bool>,
    /// the model to use for this session.
    pub model: String,
    /// the messages in this session.
    pub messages: Vec<SessionMessage>,
    /// the seed for this session.
    pub seed: u64,
    /// the temperature for this session.
    pub temperature: f32,
    /// the tools available to the AI in this session.
    pub tools: Vec<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct GetItemQueryResponse {
    // base64-encoded object
    pub data: String,
}

#[derive(Deserialize, Debug)]
pub struct GetItemQueryResponseData {
    // stringified JSON object, if the item exists
    pub item: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AiBouncerWidgetData {
    pub requirements: String,
}

impl Session {
    pub fn path(dao: &str, address: &str) -> String {
        format!("./sessions/{}_{}.json", dao, address)
    }

    pub async fn load(
        reactor: &Reactor,
        env: &Env,
        dao: &str,
        address: &str,
        model_if_new: &str,
    ) -> Result<Self, String> {
        let path = Self::path(dao, address);
        let path = std::path::Path::new(&path);

        let session = if path.exists() {
            let content = std::fs::read_to_string(path)
                .map_err(|e| format!("failed to read session file: {e}"))?;

            serde_json::from_str(&content)
                .map_err(|e| format!("failed to parse session file: {e}"))?
        } else {
            std::fs::create_dir_all("./sessions")
                .map_err(|e| format!("failed to create sessions directory: {e}"))?;

            // query the DAO to get the requirements
            // the encoded base64 is: {"get_item":{"key":"widget:ai_bouncer"}}
            let requirements_query = format!(
                "{}/cosmwasm/wasm/v1/contract/{}/smart/eyJnZXRfaXRlbSI6eyJrZXkiOiJ3aWRnZXQ6YWlfYm91bmNlciJ9fQ==",
                env.lcd,
                dao,
            );

            let req = Request::get(&requirements_query)?;
            let res = reactor.send(req).await?;

            let raw = String::from_utf8(res.body.clone());
            dbg!("get item response: {:#?}", &raw);

            if res.status != 200 {
                return Err(format!("get item unexpected status code: {}", res.status));
            }

            let requirements = match res.json::<GetItemQueryResponse>() {
                Ok(response) => {
                    let data = BASE64_STANDARD
                        .decode(&response.data)
                        .map_err(|e| format!("failed to decode base64: {e}"))?;

                    let data: GetItemQueryResponseData = serde_json::from_slice(&data)
                        .map_err(|e| format!("failed to parse item response: {e}"))?;

                    if let Some(item) = data.item {
                        let data: AiBouncerWidgetData = serde_json::from_str(&item)
                            .map_err(|e| format!("failed to parse item: {e}"))?;

                        Ok(Some(data.requirements))
                    } else {
                        Ok(None)
                    }
                }
                Err(e) => Err(format!(
                    "get item response parsing error ({e}). body: {raw:?}"
                )),
            }?;

            let system_message = match requirements {
                Some(requirements) => format!(
                    "You are a helpful bouncer deciding who is allowed to join an organization. Converse with the applicant, and make a decision using the make_decision tool only once you are confident you have enough information. Do not make a decision too early. If you cannot make a decision, ask follow up questions. The members of the organization provided the following requirements/guidelines for assessing applicants:\n\n{requirements}\n\nDecide based on the information you receive and the details provided by the organization."
                ),
                None => "You are a bouncer deciding who is allowed to join an organization. Converse with the applicant, and make a decision using the make_decision tool only once you are confident you have enough information. Do not make a decision too early. If you cannot make a decision, ask follow up questions. The members of the organization have not provided any guidelines, so use your best judgement.".to_string(),
            };

            Session {
                dao: dao.to_string(),
                address: address.to_string(),
                decision: None,
                model: model_if_new.to_string(),
                messages: vec![SessionMessage {
                    role: "system".to_string(),
                    content: system_message,
                    decision: None,
                }],
                seed: 42,
                temperature: 0.4,
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
            }
        };

        // ensure the session does not yet have a decision
        if session.decision.is_some() {
            return Err("a decision has already been made".to_string());
        }

        Ok(session)
    }

    /// ensure the message_id is the next user message
    pub fn validate_message_id(&self, message_id: u16) -> Result<(), String> {
        let user_messages = self.messages.iter().filter(|m| m.role == "user").count();

        if message_id != user_messages as u16 {
            return Err(format!(
                "message_id mismatch: expected {} but got {}",
                user_messages, message_id
            ));
        }

        Ok(())
    }

    pub fn add_user_message(&mut self, content: &str) {
        self.messages.push(SessionMessage {
            role: "user".to_string(),
            content: content.to_string(),
            decision: None,
        });
    }

    pub fn add_ai_message(&mut self, content: &str, decision: Option<bool>) {
        if let Some(decision) = decision {
            self.decision = Some(decision);
        }

        self.messages.push(SessionMessage {
            role: "assistant".to_string(),
            content: content.to_string(),
            decision,
        });
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::path(&self.dao, &self.address);
        std::fs::write(path, serde_json::to_string(&self).unwrap())
            .map_err(|e| format!("failed to write session file: {e}"))
    }
}
