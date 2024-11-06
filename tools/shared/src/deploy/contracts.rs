use std::str::FromStr;

use anyhow::{anyhow, bail, Context, Result};
use cosmwasm_std::Decimal;
use lavs_apis::time::Duration;
use lavs_task_queue::msg::{Requestor, TimeoutInfo};
use layer_climb::prelude::*;

use crate::wasmatic::load_wasmatic_addresses;

#[derive(Debug, Clone)]
pub struct CodeIds {
    pub mock_operators: u64,
    pub task_queue: u64,
    pub verifier_simple: u64,
    pub verifier_oracle: u64,
}

#[derive(Debug, Clone)]
pub struct DeployContractAddrs {
    pub operator: Address,
    pub task_queue: Address,
    pub verifier: Address,
}

#[derive(Debug, Clone)]
pub struct DeployContractArgs {
    pub code_ids: CodeIds,
    pub operators: Vec<lavs_mock_operators::msg::InstantiateOperator>,
    pub requestor: Requestor,
    pub task_timeout: TimeoutInfo,
    pub owner: Option<String>,
    pub verifier_mode: DeployVerifierMode,
}

#[derive(Clone, PartialEq, Copy, Debug)]
pub enum DeployContractArgsVerifierMode {
    Simple,
    Oracle,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeployContractArgsRequestor {
    Deployer,
    Fixed(String),
    Payment { amount: u128, denom: Option<String> },
}

impl Default for DeployContractArgsRequestor {
    fn default() -> Self {
        DeployContractArgsRequestor::Payment {
            amount: 5_000,
            // implementation fills out chain_config.gas_denom in case of None
            denom: None,
        }
    }
}
impl DeployContractArgs {
    #[allow(clippy::too_many_arguments)]
    pub async fn parse(
        http_client: reqwest::Client,
        signing_client: SigningClient,
        endpoints: Vec<String>,
        code_ids: CodeIds,
        owner: Option<String>,
        task_timeout: Duration,
        required_voting_percentage: u32,
        threshold_percentage: Option<Decimal>,
        allowed_spread: Option<Decimal>,
        slashable_spread: Option<Decimal>,
        operators: Vec<String>,
        requestor: DeployContractArgsRequestor,
        mode: DeployContractArgsVerifierMode,
    ) -> Result<DeployContractArgs> {
        if operators.is_empty() {
            bail!("At least one operator must be specified");
        }

        let mut instantiate_operators = vec![];
        for s in operators.into_iter() {
            let mut parts = s.split(':');
            let addr_str = parts.next().unwrap().to_string();
            match addr_str.as_str() {
                "wasmatic" => {
                    let wasmatic_addresses: Vec<Address> =
                        load_wasmatic_addresses(http_client.clone(), &endpoints)
                            .await?
                            .into_iter()
                            .map(|addr| {
                                signing_client
                                    .querier
                                    .chain_config
                                    .parse_address(&addr)
                                    .unwrap()
                            })
                            .collect();
                    for addr in wasmatic_addresses {
                        let voting_power = parts.next().unwrap_or("1").parse()?;
                        instantiate_operators.push(
                            lavs_mock_operators::msg::InstantiateOperator::new(
                                addr.to_string(),
                                voting_power,
                            ),
                        );
                    }
                }
                _ => {
                    let addr = signing_client
                        .querier
                        .chain_config
                        .parse_address(&addr_str)?;
                    let voting_power = parts.next().unwrap_or("1").parse()?;
                    instantiate_operators.push(lavs_mock_operators::msg::InstantiateOperator::new(
                        addr.to_string(),
                        voting_power,
                    ));
                }
            }
        }

        let requestor = match requestor {
            DeployContractArgsRequestor::Deployer => {
                Requestor::Fixed(signing_client.addr.to_string())
            }
            DeployContractArgsRequestor::Fixed(s) => Requestor::Fixed(
                signing_client
                    .querier
                    .chain_config
                    .parse_address(&s)?
                    .to_string(),
            ),
            DeployContractArgsRequestor::Payment { amount, denom } => {
                Requestor::OpenPayment(cosmwasm_std::coin(
                    amount,
                    denom.unwrap_or(signing_client.querier.chain_config.gas_denom.clone()),
                ))
            }
        };

        let task_timeout = TimeoutInfo::new(task_timeout);

        Ok(Self {
            code_ids,
            operators: instantiate_operators,
            requestor,
            owner,
            task_timeout,
            verifier_mode: match mode {
                DeployContractArgsVerifierMode::Simple => DeployVerifierMode::Simple {
                    required_voting_percentage,
                },
                DeployContractArgsVerifierMode::Oracle => DeployVerifierMode::Oracle {
                    required_voting_percentage,
                    threshold_percentage: threshold_percentage
                        .context("Threshold percentage is required for oracle verifier")?,
                    allowed_spread: allowed_spread
                        .context("Allowed spread is required for oracle verifier")?,
                    slashable_spread: slashable_spread
                        .context("Slashable spread is required for oracle verifier")?,
                },
            },
        })
    }
}

#[derive(Debug, Clone)]
pub enum DeployVerifierMode {
    Simple {
        required_voting_percentage: u32,
    },
    Oracle {
        required_voting_percentage: u32,
        threshold_percentage: Decimal,
        allowed_spread: Decimal,
        slashable_spread: Decimal,
    },
}

impl DeployContractAddrs {
    pub async fn run(client: SigningClient, args: DeployContractArgs) -> Result<Self> {
        let DeployContractArgs {
            code_ids,
            operators,
            requestor,
            owner,
            task_timeout,
            verifier_mode,
        } = args;

        let (operators_addr, tx_resp) = client
            .contract_instantiate(
                client.addr.clone(),
                code_ids.mock_operators,
                "Mock Operators",
                &lavs_mock_operators::msg::InstantiateMsg { operators },
                vec![],
                None,
            )
            .await?;

        tracing::debug!("Mock Operators Tx Hash: {}", tx_resp.txhash);
        tracing::debug!("Mock Operators Address: {}", operators_addr);

        let (verifier_addr, tx_resp) = match verifier_mode {
            DeployVerifierMode::Simple {
                required_voting_percentage,
            } => {
                client
                    .contract_instantiate(
                        client.addr.clone(),
                        code_ids.verifier_simple,
                        "Verifier Simple",
                        &lavs_verifier_simple::msg::InstantiateMsg {
                            operator_contract: operators_addr.to_string(),
                            required_percentage: required_voting_percentage,
                        },
                        vec![],
                        None,
                    )
                    .await?
            }
            DeployVerifierMode::Oracle {
                required_voting_percentage,
                threshold_percentage,
                allowed_spread,
                slashable_spread,
            } => {
                client
                    .contract_instantiate(
                        client.addr.clone(),
                        code_ids.verifier_oracle,
                        "Oracle Price Verifier",
                        &lavs_oracle_verifier::msg::InstantiateMsg {
                            operator_contract: operators_addr.to_string(),
                            required_percentage: required_voting_percentage,
                            threshold_percentage,
                            allowed_spread,
                            slashable_spread,
                        },
                        vec![],
                        None,
                    )
                    .await?
            }
        };

        tracing::debug!("Verifier Tx Hash: {}", tx_resp.txhash);
        tracing::debug!("Verifier Address: {}", verifier_addr);

        let (task_queue_addr, tx_resp) = client
            .contract_instantiate(
                client.addr.clone(),
                code_ids.task_queue,
                "Task Queue",
                &lavs_task_queue::msg::InstantiateMsg {
                    requestor,
                    timeout: task_timeout,
                    verifier: verifier_addr.to_string(),
                    owner,
                    task_specific_whitelist: None,
                },
                vec![],
                None,
            )
            .await?;

        tracing::debug!("Task Queue Tx Hash: {}", tx_resp.txhash);
        tracing::debug!("Task Queue Address: {}", task_queue_addr);

        Ok(Self {
            operator: operators_addr,
            task_queue: task_queue_addr,
            verifier: verifier_addr,
        })
    }
}

/// Supporting impls needed for custom types
impl FromStr for DeployContractArgsRequestor {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s == "deployer" {
            Ok(DeployContractArgsRequestor::Deployer)
        } else if s.starts_with("payment(") && s.ends_with(')') {
            let inner = &s[8..s.len() - 1]; // Extract content inside parentheses
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

            match parts.len() {
                1 => {
                    let amount = parts[0]
                        .parse::<u128>()
                        .map_err(|_| anyhow!("invalid amount"))?;
                    Ok(DeployContractArgsRequestor::Payment {
                        amount,
                        denom: None,
                    })
                }
                2 => {
                    let amount = parts[0]
                        .parse::<u128>()
                        .map_err(|_| anyhow!("invalid amount"))?;
                    let denom = Some(parts[1].to_string());
                    Ok(DeployContractArgsRequestor::Payment { amount, denom })
                }
                _ => Err(anyhow!("invalid format")),
            }
        } else if s.starts_with("fixed(") && s.ends_with(')') {
            let inner = &s[6..s.len() - 1]; // Extract content inside parentheses
            Ok(DeployContractArgsRequestor::Fixed(inner.trim().to_string()))
        } else {
            Err(anyhow!("unknown variant"))
        }
    }
}

impl std::fmt::Display for DeployContractArgsRequestor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeployContractArgsRequestor::Payment { amount, denom } => match denom {
                Some(denom) => write!(f, "payment({}, {})", amount, denom),
                None => write!(f, "payment({})", amount),
            },
            DeployContractArgsRequestor::Fixed(identifier) => {
                write!(f, "fixed({})", identifier)
            }
            DeployContractArgsRequestor::Deployer => {
                write!(f, "deployer")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_deployer() {
        let input = "deployer";
        let result = DeployContractArgsRequestor::from_str(input).unwrap();
        assert_eq!(result, DeployContractArgsRequestor::Deployer);
    }

    #[test]
    fn test_parse_payment_amount_only() {
        let input = "payment(200)";
        let result = DeployContractArgsRequestor::from_str(input).unwrap();
        assert_eq!(
            result,
            DeployContractArgsRequestor::Payment {
                amount: 200,
                denom: None,
            }
        );
    }

    #[test]
    fn test_parse_payment_amount_and_denom() {
        let input = "payment(300, USD)";
        let result = DeployContractArgsRequestor::from_str(input).unwrap();
        assert_eq!(
            result,
            DeployContractArgsRequestor::Payment {
                amount: 300,
                denom: Some("USD".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_payment_with_whitespace() {
        let input = " payment( 400 ,  EUR ) ";
        let result = DeployContractArgsRequestor::from_str(input).unwrap();
        assert_eq!(
            result,
            DeployContractArgsRequestor::Payment {
                amount: 400,
                denom: Some("EUR".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_fixed() {
        let input = "fixed(my_identifier)";
        let result = DeployContractArgsRequestor::from_str(input).unwrap();
        assert_eq!(
            result,
            DeployContractArgsRequestor::Fixed("my_identifier".to_string())
        );
    }

    #[test]
    fn test_parse_fixed_with_whitespace() {
        let input = " fixed( my_identifier ) ";
        let result = DeployContractArgsRequestor::from_str(input).unwrap();
        assert_eq!(
            result,
            DeployContractArgsRequestor::Fixed("my_identifier".to_string())
        );
    }

    #[test]
    fn test_parse_invalid_variant() {
        let input = "unknown(123)";
        let result = DeployContractArgsRequestor::from_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_amount() {
        let input = "payment(not_a_number)";
        let result = DeployContractArgsRequestor::from_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_format_extra_fields() {
        let input = "payment(100, USD, extra)";
        let result = DeployContractArgsRequestor::from_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_format_no_parentheses() {
        let input = "payment100, USD";
        let result = DeployContractArgsRequestor::from_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_display_payment_without_denom() {
        let requestor = DeployContractArgsRequestor::Payment {
            amount: 100,
            denom: None,
        };
        assert_eq!(format!("{}", requestor), "payment(100)");
    }

    #[test]
    fn test_display_payment_with_denom() {
        let requestor = DeployContractArgsRequestor::Payment {
            amount: 200,
            denom: Some("EUR".to_string()),
        };
        assert_eq!(format!("{}", requestor), "payment(200, EUR)");
    }

    #[test]
    fn test_display_fixed() {
        let requestor = DeployContractArgsRequestor::Fixed("identifier".to_string());
        assert_eq!(format!("{}", requestor), "fixed(identifier)");
    }

    #[test]
    fn test_display_deployer() {
        let requestor = DeployContractArgsRequestor::Deployer;
        assert_eq!(format!("{}", requestor), "deployer");
    }
}
