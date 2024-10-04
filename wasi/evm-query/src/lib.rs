#[allow(warnings)]
mod bindings;

use bindings::{Guest, Output, TaskQueueInput};
use layer_wasi::{block_on, Request, WasiPollable};
use serde::{Deserialize, Serialize};
use web3::contract::tokens::Tokenizable;
use web3::ethabi;
use web3::types::{H160, U256};

struct Component;

#[derive(Deserialize, Debug)]
pub struct TaskRequestData {
    pub address: H160,
}

#[derive(Serialize, Debug)]
pub struct TaskResponseData {
    pub address: H160,
    pub balance: U256,
}

#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Vec<serde_json::Value>,
    id: u64,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    result: U256,
}

impl Guest for Component {
    fn run_task(request: TaskQueueInput) -> Output {
        block_on(|reactor| async move {
            let TaskRequestData { address } = serde_json::from_slice(&request.request)
                .map_err(|e| format!("deserializing the task input data failed: {e}"))?;

            // TODO: Load these from environment variables.
            let rpc_url = "https://rpc.ankr.com/eth";
            let contract_address = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";

            // TODO make more generic as UI can encode the data and pass it in as a string
            let function = ethabi::Function {
                name: "balanceOf".to_owned(),
                inputs: vec![ethabi::Param {
                    name: "account".to_owned(),
                    kind: ethabi::ParamType::Address,
                    internal_type: None,
                }],
                outputs: vec![ethabi::Param {
                    name: "balance".to_owned(),
                    kind: ethabi::ParamType::Uint(256),
                    internal_type: None,
                }],
                constant: Some(true),
                state_mutability: ethabi::StateMutability::View,
            };
            let data = function
                .encode_input(&[address.into_token()])
                .map_err(|e| format!("Failed to encode function call: {}", e))?;

            // THIS BIT WILL SOON BE A WHOLE BUNCH NICER
            let json_rpc_request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: "eth_call".to_string(),
                params: vec![
                    serde_json::json!({
                        "to": contract_address,
                        "data": format!("0x{}", hex::encode(data))
                    }),
                    serde_json::json!("latest"),
                ],
                id: 1,
            };

            let mut req = Request::post(rpc_url)?;
            req.json(&json_rpc_request)?;

            // You can add hearders like so.
            // req.headers = vec![("x-cg-pro-api-key".to_string(), api_key.to_owned())];
            let res = reactor.send(req).await?;

            // TODO finish
            let JsonRpcResponse { result: balance } = match res.status {
                200 => res.json::<JsonRpcResponse>()?,
                429 => return Err("rate limited, price unavailable".to_string()),
                status => return Err(format!("unexpected status code: {status}")),
            };

            serde_json::to_vec(&TaskResponseData { address, balance })
                .map_err(|e| format!("Could not serialize response JSON: {}", e))
        })
    }
}

bindings::export!(Component with_types_in bindings);
