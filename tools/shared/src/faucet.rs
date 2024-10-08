use anyhow::Result;
use layer_climb::{prelude::*, proto::abci::TxResponse};

pub async fn tap_faucet(
    faucet_client: SigningClient,
    to: Address,
    amount: u128,
    denom: Option<String>,
) -> Result<TxResponse> {
    tracing::info!(
        "Balance before: {}",
        faucet_client
            .querier
            .balance(to.clone(), denom.clone())
            .await?
            .unwrap_or_default()
    );

    let tx_resp = faucet_client
        .transfer(amount, &to, denom.as_deref(), None)
        .await?;

    tracing::info!("Tapped faucet for {}, sent to {}", amount, to);
    tracing::info!(
        "Balance after: {}",
        faucet_client
            .querier
            .balance(to.clone(), denom.clone())
            .await?
            .unwrap_or_default()
    );

    Ok(tx_resp)
}
