use std::sync::LazyLock;

use dominator::ColorScheme;
use scheme::{color_scheme_signal, set_color_scheme};
use wasm_bindgen_futures::spawn_local;

use crate::{
    page::logo::LogoSvg, prelude::*, route::TaskQueueRoute, util::mixins::handle_on_click,
};

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
            }
        });
        static LOGO: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("align-items", "center")
                .style("margin-left", "1rem")
                .style("margin-bottom", "2.75rem")
                .style("gap", ".75rem")
            }
        });

        static MENU: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("gap", "1.3125rem")
                .style("align-items", "flex-start")
            }
        });

        html!("div", {
            .class(&*CONTAINER)
            .child(html!("div", {
                .class(&*MENU)
                .child(html!("div", {
                    .class(&*LOGO)
                    .child(LogoSvg::render())
                    .child(html!("div", {
                        .class(FontSize::Header.class())
                        .text("Layer AVS Toolkit")
                    }))
                }))
                .children([
                    self.render_section("Task Queue", vec![
                        Route::TaskQueue(TaskQueueRoute::AddTask),
                        Route::TaskQueue(TaskQueueRoute::ViewQueue),
                    ]),
                    self.render_section("Wasmatic", vec![
                        Route::Wasmatic(WasmaticRoute::AddApp),
                        Route::Wasmatic(WasmaticRoute::ListApps),
                        Route::Wasmatic(WasmaticRoute::TestApp),
                        Route::Wasmatic(WasmaticRoute::Info),
                    ]),
                    self.render_section("Wallet", vec![
                        Route::Wallet(WalletRoute::Faucet),
                    ]),
                    self.render_section("Contract", vec![
                        Route::Contract(ContractRoute::Upload),
                        Route::Contract(ContractRoute::Instantiate),
                        Route::Contract(ContractRoute::Execute),
                        Route::Contract(ContractRoute::Query),
                    ]),
                    self.render_section("Block", vec![
                        Route::BlockEvents,
                    ]),
                ])
            }))
            .child(render_color_scheme_toggle())
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
                .style_signal("color", ColorText::Secondary.signal())
            }
        });

        let selected_sig = Route::signal().map(clone!(routes => move |selected_route| {
            routes.iter().any(|route| selected_route == *route)
        }));

        html!("div", {
            .class(&*CONTAINER)
            .children([
                html!("div", {
                    .class([FontSize::Caption.class(), &*TITLE])
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
                .style_signal("background-color", ColorBackgroundInteractive::Selected.signal())
            }
        });

        let selected_sig = Route::signal().map(clone!(route => move |selected_route| {
            selected_route == route
        }));

        html!("div", {
            .class([&*BUTTON_BG_CLASS, &*ColorText::Body.color_class(), FontSize::Primary.class(), &*USER_SELECT_NONE])
            .class_signal([&*SELECTED, FontWeight::Bold.class()] , selected_sig)

            .text(match &route {
                Route::Wallet(wallet_route) => match wallet_route {
                    WalletRoute::Faucet => "Tap Faucet"
                },
                Route::Contract(contract_route) => match contract_route {
                    ContractRoute::Upload => "Upload",
                    ContractRoute::Instantiate => "Instantiate",
                    ContractRoute::Execute => "Execute",
                    ContractRoute::Query => "Query",
                },
                Route::Wasmatic(wasmatic_route) => match wasmatic_route {
                    WasmaticRoute::AddApp => "Add App",
                    WasmaticRoute::ListApps => "List Apps",
                    WasmaticRoute::Info => "Info",
                    WasmaticRoute::TestApp => "Test App",
                },
                Route::TaskQueue(task_queue_route) => match task_queue_route {
                    TaskQueueRoute::AddTask => "Add Task",
                    TaskQueueRoute::ViewQueue => "View Queue",
                },
                Route::BlockEvents => "Events",
                _ => unreachable!()
            })
            .apply(handle_on_click(move || {
                route.go_to_url();
            }))
        })
    }
}

fn render_color_scheme_toggle() -> Dom {
    let current = Arc::new(Mutex::new(None));

    static CONTAINER: LazyLock<String> = LazyLock::new(|| {
        class! {
            .style("padding", "1rem")
        }
    });
    html!("div", {
        .future(color_scheme_signal().for_each(clone!(current => move |scheme| {
            clone!(current => async move {
                *current.lock().unwrap_ext() = Some(scheme);
            })
        })))
        .class(&*CONTAINER)
        .child(Button::new()
            .with_text("Toggle Color Scheme")
            .with_on_click(clone!(current => move || {
                let new_scheme = match *current.lock().unwrap_ext() {
                    Some(ColorScheme::Light) => ColorScheme::Dark,
                    _ => ColorScheme::Light,
                };

                set_color_scheme(new_scheme);
            }))
            .render()
        )
    })
}
