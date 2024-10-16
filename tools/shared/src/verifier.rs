use anyhow::Result;
use lavs_verifier_simple::msg::{ConfigResponse, QueryMsg};
use layer_climb::prelude::*;

pub struct SimpleVerifierQuerier {
    pub contract_addr: Address,
    pub query_client: QueryClient,
}

impl SimpleVerifierQuerier {
    pub async fn new(query_client: QueryClient, contract_addr: Address) -> Result<Self> {
        Ok(Self {
            query_client,
            contract_addr,
        })
    }

    pub async fn config(&self) -> Result<ConfigResponse> {
        self.query_client
            .contract_smart(&self.contract_addr, &QueryMsg::Config {})
            .await
    }

    pub async fn operator_addr(&self) -> Result<Address> {
        let config = self.config().await?;
        self.query_client
            .chain_config
            .parse_address(&config.operator_contract)
    }
}
