use crate::context::AppContext;
use anyhow::Result;
use lavs_verifier_simple::msg::{ConfigResponse, QueryMsg};
use layer_climb::prelude::*;

pub struct SimpleVerifierQuerier {
    pub ctx: AppContext,
    pub contract_addr: Address,
    pub querier: QueryClient,
}

impl SimpleVerifierQuerier {
    pub async fn new(ctx: AppContext, contract_addr: Address) -> Result<Self> {
        Ok(Self {
            querier: ctx.query_client().await?,
            ctx,
            contract_addr,
        })
    }

    pub async fn config(&self) -> Result<ConfigResponse> {
        self.querier
            .contract_smart(&self.contract_addr, &QueryMsg::Config {})
            .await
    }

    pub async fn operator_addr(&self) -> Result<Address> {
        let config = self.config().await?;
        self.ctx
            .chain_config()?
            .parse_address(&config.operator_contract)
    }
}
