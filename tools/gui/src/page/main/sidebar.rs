use std::sync::LazyLock;

use wasm_bindgen_futures::spawn_local;

use crate::{prelude::*, util::mixins::handle_on_click};

pub struct Sidebar {}

impl Sidebar {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }

    pub fn render(self: Arc<Self>) -> Dom {
        static CONTAINER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("margin-top", "1rem")
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("gap", "1.3125rem")
                .style("align-items", "flex-start")
            }
        });

        html!("div", {
            .class(&*CONTAINER)
            .children([
                self.render_section("Wallet", vec![
                    Route::WalletFaucet,
                ]),
                self.render_section("Contract", vec![
                    Route::ContractUpload,
                    Route::ContractInstantiate,
                    Route::ContractExecute,
                    Route::ContractQuery,
                ]),
                self.render_section("Block", vec![
                    Route::BlockEvents,
                ]),
            ])
        })
    }

    fn render_section(self: &Arc<Self>, title: &str, routes: Vec<Route>) -> Dom {
        static CONTAINER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("width", "100%")
                .style("display", "flex")
                .style("flex-direction", "column")
            }
        });
        static TITLE: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("margin-left", "1rem")
            }
        });

        let selected_sig = Route::signal().map(clone!(routes => move |selected_route| {
            routes.iter().any(|route| selected_route == *route)
        }));

        html!("div", {
            .class(&*CONTAINER)
            .children([
                html!("div", {
                    .class([&*TEXT_SIZE_XLG, &*TITLE, Color::Grey.class()])
                    .text(title)
                }),
                html!("div", {
                    .style("width", "100%")
                    .children(routes.into_iter().map(|route| {
                        self.render_button(route)
                    }).collect::<Vec<Dom>>())
                })
            ])
        })
    }

    fn render_button(self: &Arc<Self>, route: Route) -> Dom {
        static BUTTON_BG_CLASS: LazyLock<String> = LazyLock::new(|| {
            class! {
                    .style("cursor", "pointer")
                    .style("display", "flex")
                    .style("justify-content", "flex-start")
                    .style("align-items", "center")
                    .style("gap", "1.5rem")
                    .style("width", "100%")
                    .style("padding", "1.25rem 2.88rem")
            }
        });

        static SELECTED: LazyLock<String> = LazyLock::new(|| {
            class! {
                    .style("background-color", Color::GreyAlt1.hex_str())
                    .style("color", Color::Accent.hex_str())
            }
        });

        let selected_sig = Route::signal().map(clone!(route => move |selected_route| {
            selected_route == route
        }));

        html!("div", {
            .class([&*BUTTON_BG_CLASS, &*TEXT_SIZE_XLG, &*USER_SELECT_NONE])
            .class_signal([&*SELECTED, &*TEXT_WEIGHT_BOLD] , selected_sig)

            .text(match route {
                Route::WalletFaucet => "Tap Faucet",
                Route::ContractUpload => "Contract Upload",
                Route::ContractInstantiate => "Contract Instantiate",
                Route::ContractExecute => "Contract Execute",
                Route::ContractQuery => "Contract Query",
                Route::BlockEvents => "Block Events",
                _ => unreachable!(),
            })
            .apply(handle_on_click(move || {
                route.go_to_url();
            }))
        })
    }
}
