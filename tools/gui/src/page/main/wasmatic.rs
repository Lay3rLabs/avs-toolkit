use std::collections::{hash_map::Entry, HashMap};

use crate::prelude::*;
mod add_app;
mod app_test;
mod info;
mod list_apps;
mod run;

pub use add_app::*;
pub use app_test::*;
use avs_toolkit_shared::wasmatic::{self, AppInfo};
pub use info::*;
pub use list_apps::*;
pub use run::*;

pub(super) struct AppEntry {
    pub app: AppInfo,
    pub endpoints: Vec<String>,
}
pub(super) async fn get_apps() -> Result<Vec<Arc<AppEntry>>> {
    let mut apps: HashMap<String, AppEntry> = HashMap::new();

    let mut responses = wasmatic::all_apps(
        http_client(),
        CONFIG.chain_info().unwrap_ext().wasmatic.endpoints.clone(),
        |apps| {
            log::info!("apps: {:?}", apps);
        },
    )
    .await?;

    for response in responses.drain(..) {
        for app in response.app.apps {
            match apps.entry(app.name.clone()) {
                Entry::Occupied(mut entry) => {
                    let entry: &mut AppEntry = entry.get_mut();
                    if entry.app != app {
                        return Err(anyhow!("App with the same name but different data"));
                    }
                    entry.endpoints.push(response.endpoint.clone());
                }
                Entry::Vacant(entry) => {
                    entry.insert(AppEntry {
                        app,
                        endpoints: vec![response.endpoint.clone()],
                    });
                }
            }
        }
    }

    Ok(apps.into_iter().map(|(_, v)| Arc::new(v)).collect())
}
