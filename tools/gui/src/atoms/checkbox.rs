use std::pin::Pin;

use crate::prelude::*;

pub struct Checkbox {
    selected_signal: Option<Pin<Box<dyn Signal<Item = bool>>>>,
    on_click: Option<Arc<dyn Fn()>>,
    label: Option<String>,
}

impl Checkbox {
    pub fn new() -> Self {
        Self {
            selected_signal: None,
            label: None,
            on_click: None,
        }
    }

    pub fn with_selected_signal(
        mut self,
        selected_signal: impl Signal<Item = bool> + 'static,
    ) -> Self {
        self.selected_signal = Some(Box::pin(selected_signal));
        self
    }

    pub fn with_label(mut self, label: impl ToString) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn with_on_click(mut self, on_click: impl Fn() + 'static) -> Self {
        self.on_click = Some(Arc::new(on_click));
        self
    }

    pub fn render(self) -> Dom {
        static CONTAINER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("gap", ".5rem")
                .style("align-items", "center")
            }
        });
        static SVG: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("width", "1.5rem")
                .style("height", "1.5rem")
            }
        });

        let Self {
            selected_signal,
            label,
            on_click,
        } = self;

        html!("div", {
            .class([&*CONTAINER, &*CURSOR_POINTER, &*USER_SELECT_NONE])
            .apply_if(on_click.is_some(), |dom| {
                let on_click = on_click.unwrap_ext();
                dom.event(clone!(on_click => move |_: events::Click| {
                    on_click();
                }))
            })
            .apply_if(label.is_some(), |dom| {
                dom.child(html!("div", {
                    .class(FontSize::Body.class())
                    .text(&label.unwrap_ext())
                }))
            })
            .child(svg!("svg", {
                .class(&*SVG)
                .attrs!{
                    "xmlns": "http://www.w3.org/2000/svg",
                    "viewBox": "0 0 41 41",
                    "fill": "none",
                }
                .apply_if(selected_signal.is_some(), |dom| {
                    dom.child_signal(selected_signal.unwrap_ext().map(|selected| {
                        Some(if selected {
                            svg!("path", {
                                .attr("d", "M34.2919 0.447449H6.59963C4.96808 0.44923 3.40386 1.09815 2.25017 2.25184C1.09649 3.40552 0.447567 4.96974 0.445786 6.60129V34.2936C0.447567 35.9252 1.09649 37.4894 2.25017 38.6431C3.40386 39.7967 4.96808 40.4457 6.59963 40.4474H34.2919C35.9235 40.4457 37.4877 39.7967 38.6414 38.6431C39.7951 37.4894 40.444 35.9252 40.4458 34.2936V6.60129C40.444 4.96974 39.7951 3.40552 38.6414 2.25184C37.4877 1.09815 35.9235 0.44923 34.2919 0.447449ZM30.8544 13.7446L17.9314 29.1292C17.7896 29.298 17.6132 29.4344 17.4141 29.5292C17.2151 29.6239 16.998 29.6747 16.7775 29.6782H16.7516C16.5359 29.6781 16.3227 29.6327 16.1257 29.5449C15.9288 29.4571 15.7525 29.3289 15.6083 29.1686L10.0698 23.0148C9.92917 22.8656 9.81975 22.6898 9.748 22.4977C9.67626 22.3056 9.64363 22.1011 9.65204 21.8963C9.66045 21.6914 9.70973 21.4903 9.79697 21.3047C9.88422 21.1192 10.0077 20.9529 10.1601 20.8158C10.3125 20.6786 10.4908 20.5733 10.6845 20.5061C10.8782 20.4388 11.0833 20.4109 11.288 20.4241C11.4926 20.4372 11.6925 20.4912 11.876 20.5827C12.0595 20.6742 12.2228 20.8015 12.3564 20.9571L16.7112 25.7955L28.4987 11.7657C28.7631 11.46 29.1372 11.2707 29.5401 11.2386C29.943 11.2064 30.3423 11.3342 30.6518 11.5941C30.9613 11.8541 31.156 12.2254 31.1939 12.6278C31.2319 13.0302 31.1099 13.4314 30.8544 13.7446Z")
                                .attr_signal("fill", ColorTextInteractive::Valid.signal())
                            })
                        } else {
                            svg!("rect", {
                                .attrs!{
                                    "rx": "5",
                                    "x": "1.4458",
                                    "y": "1.44727",
                                    "width": "38",
                                    "height": "38",
                                    "stroke-width": "2",
                                }
                                .attr_signal("stroke", ColorBorder::Base.signal())
                            })
                        })
                    }))
                })
            }))
        })
    }
}
