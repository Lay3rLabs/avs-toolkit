use crate::{
    client::{has_signing_client, signing_client},
    page::{landing::LandingUi, main::MainUi, notfound::NotFoundUi},
    prelude::*,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Route {
    Landing,
    WalletFaucet,
    ContractUpload,
    ContractInstantiate,
    ContractExecute,
    ContractQuery,
    BlockEvents,
    NotFound,
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
            ["wallet", "faucet"] => Self::WalletFaucet,
            ["contract", "upload"] => Self::ContractUpload,
            ["contract", "instantiate"] => Self::ContractInstantiate,
            ["contract", "execute"] => Self::ContractExecute,
            ["contract", "query"] => Self::ContractQuery,
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
            Route::WalletFaucet => "wallet/faucet".to_string(),
            Route::ContractUpload => "contract/upload".to_string(),
            Route::ContractInstantiate => "contract/instantiate".to_string(),
            Route::ContractExecute => "contract/execute".to_string(),
            Route::ContractQuery => "contract/query".to_string(),
            Route::BlockEvents => "block/events".to_string(),
            Route::NotFound => "404".to_string(),
        };
        write!(f, "{}", s)
    }
}

pub fn render() -> Dom {
    html!("div", {
        .style("width", "100%")
        .style("height", "100%")
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
