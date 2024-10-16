use std::pin::Pin;

use crate::prelude::*;
use futures::{pin_mut, Stream, StreamExt};
use futures_signals::{signal, signal_vec};
use layer_climb::querier::stream::BlockEvents;

pub struct BlockEventsUi {
    pub error: Mutable<Option<String>>,
    pub stream_ready: Mutable<bool>,
    pub only_blocks_with_events: Mutable<bool>,
}

impl BlockEventsUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            error: Mutable::new(None),
            stream_ready: Mutable::new(false),
            only_blocks_with_events: Mutable::new(false),
        })
    }

    pub fn render(self: &Arc<Self>) -> Dom {
        html!("div", {
            .children([
                self.render_header(),
                self.render_list()
            ])
        })
    }

    fn render_header(self: &Arc<Self>) -> Dom {
        let state = self;

        static HEADER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("margin-bottom", "1rem")
            }
        });

        html!("div", {
            .class(&*HEADER)
            .child(Checkbox::new()
                .with_label("Only blocks with events")
                .with_selected_signal(state.only_blocks_with_events.signal())
                .with_on_click(clone!(state => move || {
                    state.only_blocks_with_events.set_neq(!state.only_blocks_with_events.get());
                }))
                .render()
            )
        })
    }
    fn render_list(self: &Arc<Self>) -> Dom {
        let state = self;
        let stream = query_client().stream_block_events(None);

        html!("div", {
            .child_signal(signal::from_future(stream).map(clone!(state => move |block_events| {
                match block_events {
                    Some(Ok(stream)) => {
                        Some(html!("div", {
                            .children_signal_vec(signal_vec::from_stream(stream).map(clone!(state => move |block_events| {
                                match block_events {
                                    Ok(block_events) => {
                                        state.render_block_events(block_events)
                                    },
                                    Err(err) => {
                                        html!("div", {
                                            .class([FontSize::Body.class(), &*COLOR_TEXT_INTERACTIVE_ERROR])
                                            .text("Error fetching block events")
                                        })
                                    }
                                }
                            })))
                        }))
                    },
                    Some(Err(err)) => {
                        Some(html!("div", {
                            .class([FontSize::Body.class(), &*COLOR_TEXT_INTERACTIVE_ERROR])
                            .text("Error fetching block events")
                        }))
                    },
                    None => None
                }
            })))
        })
    }

    fn render_block_events(self: &Arc<Self>, block_events: BlockEvents) -> Dom {
        let state = self;

        static CONTAINER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("gap", "1rem")
                .style("border", "1px solid black")
                .style("padding", "1rem")
            }
        });
        static HEADER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("gap", "10px")
                .style("justify-content", "space-between")
            }
        });
        static EVENT_TYPE: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("gap", ".3rem")
                .style("margin-top", "1rem")
                .style("margin-bottom", ".5rem")
            }
        });
        static EVENT_ATTRS: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("margin-top", ".5rem")
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("gap", ".3rem")
                .style("margin-left", "1rem")
            }
        });
        static EVENT_ATTR: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("gap", ".3rem")
            }
        });
        let expanded = Mutable::new(false);

        let event_len = block_events.events.len();

        html!("div", {
            .class(&*CONTAINER)
            .class(FontSize::Body.class())
            .style_signal("display", state.only_blocks_with_events.signal().map(move |only_blocks_with_events| {
                if !only_blocks_with_events || event_len > 0 {
                    "block"
                } else {
                    "none"
                }
            }))
            .child(html!("div", {
                .class(&*HEADER)
                .child(html!("div", {
                    .text(&format!("Block #{}", block_events.height))
                }))
                .child(html!("div", {
                    .class([&*ColorText::Brand.color_class(), &*CURSOR_POINTER])
                    .text_signal(expanded.signal().map(move |is_expanded| {
                        if is_expanded {
                            format!("Hide events ({event_len})")
                        } else {
                            format!("Show events ({event_len})")
                        }
                    }))
                    .event(clone!(expanded => move |_: events::Click| {
                        expanded.set_neq(!expanded.get());
                    }))
                }))
            }))
            .child(html!("div", {
                .style_signal("display", expanded.signal().map(|expanded| {
                    if expanded {
                        "block"
                    } else {
                        "none"
                    }
                }))
                .children(block_events.events.iter().map(|event| {
                    html!("div", {
                        .child(html!("div", {
                            .class(&*EVENT_TYPE)
                            .child(html!("span", {
                                .text("Event type")
                            }))
                            .child(html!("span", {
                                .class([ColorText::Body.color_class(), FontWeight::Bold.class()])
                                .text(&event.kind)
                            }))
                        }))
                        .child(html!("div", {
                            .text("Attributes:")
                        }))
                        .child(html!("div", {
                            .class(&*EVENT_ATTRS)
                            .children(event.attributes.iter().map(|attr| {
                                html!("div", {
                                    .class(&*EVENT_ATTR)
                                    .child(html!("span", {
                                        .class([ColorText::Body.color_class(), FontWeight::Bold.class()])
                                        .text(&format!("{}:", attr.key_str().unwrap_or_default()))
                                    }))
                                    .child(html!("span", {
                                        .text(attr.value_str().unwrap_or_default())
                                    }))
                                })
                            }))
                        }))
                    })
                }))
            }))
        })
    }
}
