use anyhow::Result;
use layer_climb::{prelude::*, proto::abci::TxResponse};

use crate::context::AppContext;

pub async fn tap_faucet(
    ctx: &AppContext,
    to: Address,
    amount: u128,
    denom: Option<String>,
) -> Result<TxResponse> {
    tracing::info!(
        "Balance before: {}",
        ctx.query_client()
            .await?
            .balance(to.clone(), denom.clone())
            .await?
            .unwrap_or_default()
    );

    let faucet = ctx.faucet_client().await?;

    let tx_resp = faucet.transfer(amount, &to, denom.as_deref(), None).await?;

    tracing::info!("Tapped faucet for {}, sent to {}", amount, to);
    tracing::info!(
        "Balance after: {}",
        ctx.query_client()
            .await?
            .balance(to.clone(), denom.clone())
            .await?
            .unwrap_or_default()
    );

    Ok(tx_resp)
}
