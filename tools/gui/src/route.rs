use crate::{
    client::{has_signing_client, signing_client},
    page::{landing::LandingUi, main::MainUi, notfound::NotFoundUi},
    prelude::*,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Route {
    Landing,
    Wallet(WalletRoute),
    Contract(ContractRoute),
    Wasmatic(WasmaticRoute),
    TaskQueue(TaskQueueRoute),
    BlockEvents,
    NotFound,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WalletRoute {
    Faucet,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContractRoute {
    Upload,
    Instantiate,
    Execute,
    Query,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WasmaticRoute {
    AddApp,
    ListApps,
    Info,
    TestApp,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskQueueRoute {
    AddTask,
    ViewQueue,
}

impl Route {
    pub fn from_url(url: &str, root_path: &str) -> Self {
        let url = web_sys::Url::new(url).unwrap();
        let paths = url.pathname();

        let paths = paths
            .split('/')
            .into_iter()
            .skip(if CONFIG.root_path.is_empty() { 1 } else { 2 })
            // skip all the roots (1 for the domain, 1 for each part of root path)
            //.skip(root_path.chars().filter(|c| *c == '/').count() + 1)
            .collect::<Vec<_>>();

        let paths = paths.as_slice();

        let route = match paths {
            [""] => Self::Landing,
            ["/"] => Self::Landing,
            ["wallet", wallet_route] => match *wallet_route {
                "faucet" => Self::Wallet(WalletRoute::Faucet),
                _ => Self::NotFound,
            },
            ["contract", contract_route] => match *contract_route {
                "upload" => Self::Contract(ContractRoute::Upload),
                "instantiate" => Self::Contract(ContractRoute::Instantiate),
                "execute" => Self::Contract(ContractRoute::Execute),
                "query" => Self::Contract(ContractRoute::Query),
                _ => Self::NotFound,
            },
            ["wasmatic", wasmatic_route] => match *wasmatic_route {
                "add-app" => Self::Wasmatic(WasmaticRoute::AddApp),
                "list-apps" => Self::Wasmatic(WasmaticRoute::ListApps),
                "info" => Self::Wasmatic(WasmaticRoute::Info),
                "test-app" => Self::Wasmatic(WasmaticRoute::TestApp),
                _ => Self::NotFound,
            },
            ["task-queue", task_queue_route] => match *task_queue_route {
                "add-task" => Self::TaskQueue(TaskQueueRoute::AddTask),
                "view-queue" => Self::TaskQueue(TaskQueueRoute::ViewQueue),
                _ => Self::NotFound,
            },
            ["block", "events"] => Self::BlockEvents,
            _ => Self::NotFound,
        };

        route
    }

    pub fn link(&self) -> String {
        let domain = "";

        if CONFIG.root_path.is_empty() {
            format!("{}/{}", domain, self.to_string())
        } else {
            format!("{}/{}/{}", domain, CONFIG.root_path, self.to_string())
        }
    }
    pub fn go_to_url(&self) {
        dominator::routing::go_to_url(&self.link());
    }

    pub fn signal() -> impl Signal<Item = Route> {
        dominator::routing::url()
            .signal_cloned()
            .map(|url| Route::from_url(&url, CONFIG.root_path))
    }
}
impl std::fmt::Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            Route::Landing => "".to_string(),
            Route::Wallet(wallet_route) => format!("wallet/{wallet_route}"),
            Route::Contract(contract_route) => format!("contract/{contract_route}"),
            Route::Wasmatic(wasmatic_route) => format!("wasmatic/{wasmatic_route}"),
            Route::TaskQueue(task_queue_route) => format!("task-queue/{task_queue_route}"),
            Route::BlockEvents => "block/events".to_string(),
            Route::NotFound => "404".to_string(),
        };
        write!(f, "{}", s)
    }
}

impl std::fmt::Display for WalletRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            WalletRoute::Faucet => "faucet".to_string(),
        };
        write!(f, "{}", s)
    }
}

impl std::fmt::Display for ContractRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            ContractRoute::Upload => "upload".to_string(),
            ContractRoute::Instantiate => "instantiate".to_string(),
            ContractRoute::Execute => "execute".to_string(),
            ContractRoute::Query => "query".to_string(),
        };
        write!(f, "{}", s)
    }
}

impl std::fmt::Display for WasmaticRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            WasmaticRoute::AddApp => "add-app".to_string(),
            WasmaticRoute::ListApps => "list-apps".to_string(),
            WasmaticRoute::Info => "info".to_string(),
            WasmaticRoute::TestApp => "test-app".to_string(),
        };
        write!(f, "{}", s)
    }
}

impl std::fmt::Display for TaskQueueRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            TaskQueueRoute::AddTask => "add-task".to_string(),
            TaskQueueRoute::ViewQueue => "view-queue".to_string(),
        };
        write!(f, "{}", s)
    }
}

pub fn render() -> Dom {
    static CONTAINER: LazyLock<String> = LazyLock::new(|| {
        class! {
            .style("width", "100%")
            .style("height", "100%")
        }
    });
    html!("div", {
        .class(&*CONTAINER)
        .child_signal(Route::signal().map(|route| {
            match route {
                Route::Landing => Some(LandingUi::new().render()),
                Route::NotFound => Some(NotFoundUi::new().render()),
                _ => {
                    if !has_signing_client() {
                        Route::Landing.go_to_url();
                        None
                    } else {
                        Some(MainUi::new().render())
                    }
                }
            }
        }))
    })
}
