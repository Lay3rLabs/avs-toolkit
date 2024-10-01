use crate::context::AppContext;
use anyhow::Result;
use lavs_mock_operators::msg::{AllVotersResponse, QueryMsg};
use layer_climb::prelude::*;

pub struct OperatorQuerier {
    pub ctx: AppContext,
    pub contract_addr: Address,
    pub querier: QueryClient,
}

impl OperatorQuerier {
    pub async fn new(ctx: AppContext, contract_addr: Address) -> Result<Self> {
        let querier = ctx.query_client().await?;
        Ok(Self {
            ctx,
            contract_addr,
            querier,
        })
    }

    pub async fn all_operators(&self) -> Result<Vec<Operator>> {
        let all_voters: AllVotersResponse = self
            .querier
            .contract_smart(&self.contract_addr, &QueryMsg::AllVoters {})
            .await?;

        all_voters
            .voters
            .into_iter()
            .map(|v| {
                let address = self.ctx.chain_config()?.parse_address(&v.address)?;
                Ok(Operator {
                    address,
                    power: v.power.u128(),
                })
            })
            .collect()
    }
}

pub struct Operator {
    pub address: Address,
    pub power: u128,
}
