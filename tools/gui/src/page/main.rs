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
use wasmatic::{
    WasmaticAddAppUi, WasmaticInfoUi, WasmaticListAppsUi, WasmaticRunUi, WasmaticTestAppUi,
};

pub struct MainUi {}

impl MainUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }

    pub fn render(self: Arc<Self>) -> Dom {
        static CONTAINER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style_signal("background-color", ColorBackground::Base.signal())
                .style_signal("color", ColorText::Body.signal())
            }
        });

        static SIDEBAR: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("flex-shrink", "0")
                .style("min-height", "100vh")
                .style_signal("border-right", ColorBorder::Base.signal())
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
                                WasmaticRoute::AddApp => Some(WasmaticAddAppUi::new().render()),
                                WasmaticRoute::ListApps => Some(WasmaticListAppsUi::new().render()),
                                WasmaticRoute::Info => Some(WasmaticInfoUi::new().render()),
                                WasmaticRoute::TestApp => Some(WasmaticTestAppUi::new().render()),
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
