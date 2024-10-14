use anyhow::Result;
use lavs_mock_operators::msg::{AllVotersResponse, QueryMsg};
use layer_climb::prelude::*;

pub struct OperatorQuerier {
    pub contract_addr: Address,
    pub query_client: QueryClient,
}

impl OperatorQuerier {
    pub async fn new(query_client: QueryClient, contract_addr: Address) -> Result<Self> {
        Ok(Self {
            contract_addr,
            query_client,
        })
    }

    pub async fn all_operators(&self) -> Result<Vec<Operator>> {
        let all_voters: AllVotersResponse = self
            .query_client
            .contract_smart(&self.contract_addr, &QueryMsg::AllVoters {})
            .await?;

        all_voters
            .voters
            .into_iter()
            .map(|v| {
                let address = self.query_client.chain_config.parse_address(&v.address)?;
                Ok(Operator {
                    address,
                    power: v.power.u128(),
                })
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Operator {
    pub address: Address,
    pub power: u128,
}
