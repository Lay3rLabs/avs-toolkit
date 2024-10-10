mod block;
mod contract;
mod sidebar;
mod task_queue;
mod wallet;
mod wasmatic;

use crate::{prelude::*, route::TaskQueueRoute};
use block::events::BlockEventsUi;
use contract::{ContractExecuteUi, ContractInstantiateUi, ContractQueryUi, ContractUploadUi};
use task_queue::{TaskQueueAddTaskUi, TaskQueueViewQueueUi};
use wallet::faucet::WalletFaucetUi;
use wasmatic::{WasmaticDeployUi, WasmaticInfoUi, WasmaticListUi, WasmaticRunUi, WasmaticTestUi};

pub struct MainUi {}

impl MainUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }

    pub fn render(self: Arc<Self>) -> Dom {
        static CONTAINER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
            }
        });

        static SIDEBAR: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("flex-shrink", "0")
                .style("min-height", "100vh")
                .style("background-color", Color::GreyAlt2.hex_str())
            }
        });

        static CONTENT: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("flex-grow", "1")
                .style("padding", "1rem")
            }
        });

        html!("div", {
            .class(&*CONTAINER)
            .children([
                html!("div", {
                    .class(&*SIDEBAR)
                    .child(sidebar::Sidebar::new().render())
                }),
                html!("div", {
                    .class(&*CONTENT)
                    .child_signal(Route::signal().map(|route| {
                        match route {
                            Route::Wallet(wallet_route) => match wallet_route {
                                WalletRoute::Faucet => Some(WalletFaucetUi::new().render()),
                            },
                            Route::Contract(contract_route) => match contract_route {
                                ContractRoute::Upload => Some(ContractUploadUi::new().render()),
                                ContractRoute::Instantiate => Some(ContractInstantiateUi::new().render()),
                                ContractRoute::Execute => Some(ContractExecuteUi::new().render()),
                                ContractRoute::Query => Some(ContractQueryUi::new().render()),
                            },
                            Route::Wasmatic(wasmatic_route) => match wasmatic_route {
                                WasmaticRoute::Deploy => Some(WasmaticDeployUi::new().render()),
                                WasmaticRoute::List => Some(WasmaticListUi::new().render()),
                                WasmaticRoute::Info => Some(WasmaticInfoUi::new().render()),
                                WasmaticRoute::Test => Some(WasmaticTestUi::new().render()),
                            },
                            Route::TaskQueue(task_queue_route) => match task_queue_route {
                                TaskQueueRoute::AddTask => Some(TaskQueueAddTaskUi::new().render()),
                                TaskQueueRoute::ViewQueue => Some(TaskQueueViewQueueUi::new().render()),
                            },
                            Route::BlockEvents => Some(BlockEventsUi::new().render()),
                            _ => {
                                None
                            }
                        }
                    }))
                })
            ])
        })
    }
}
