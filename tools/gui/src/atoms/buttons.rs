use std::pin::Pin;

use futures_signals::signal::always;
use web_sys::HtmlElement;

use crate::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ButtonSize {
    Sm,
    Lg,
}

impl ButtonSize {
    pub fn text_size_class(self) -> &'static str {
        match self {
            Self::Sm => FontSize::ButtonSmall.class(),
            Self::Lg => FontSize::Button.class(),
        }
    }

    pub fn container_class(self) -> &'static str {
        static DEFAULT_CLASS: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("padding", "0.625rem 1.875rem")
            }
        });

        static SM_CLASS: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("padding", "0.375rem 1.25rem")
            }
        });

        match self {
            Self::Sm => &*SM_CLASS,
            _ => &*DEFAULT_CLASS,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ButtonColor {
    Regular,
    Branded,
}

static BG_COLOR_REGULAR: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("background-color", ColorBackground::Button.signal())
    }
});

static BG_COLOR_REGULAR_HOVER: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("background-color", ColorBackgroundButton::Hover.signal())
    }
});

static BG_COLOR_REGULAR_DISABLED: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("background-color", ColorBackgroundButton::Disabled.signal())
    }
});

static BG_COLOR_BRANDED_PRIMARY: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("background-color", ColorBranded::Primary.signal())
    }
});

static BG_COLOR_BRANDED_PRIMARY_HOVER: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("background-color", ColorBrandedPrimary::Hover.signal())
    }
});

static BG_COLOR_BRANDED_PRIMARY_DISABLED: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("background-color", ColorBrandedPrimary::Disabled.signal())
    }
});

static COLOR_REGULAR: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("color", ColorTextInteractiveButton::Primary.signal())
    }
});

static COLOR_REGULAR_DISABLED: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("color", ColorTextInteractiveButton::Disabled.signal())
    }
});

static COLOR_BRANDED_PRIMARY: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("color", ColorBrandedPrimary::Text.signal())
    }
});

static COLOR_BRANDED_PRIMARY_DISABLED: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("color", ColorBrandedPrimary::TextDisabled.signal())
    }
});

impl ButtonColor {
    pub fn bg_class(&self) -> &'static str {
        match self {
            Self::Regular => &BG_COLOR_REGULAR,
            Self::Branded => &BG_COLOR_BRANDED_PRIMARY,
        }
    }

    pub fn bg_hover_class(&self) -> &'static str {
        match self {
            Self::Regular => &BG_COLOR_REGULAR_HOVER,
            Self::Branded => &BG_COLOR_BRANDED_PRIMARY_HOVER,
        }
    }

    pub fn color_class(&self) -> &'static str {
        match self {
            Self::Regular => &COLOR_REGULAR,
            Self::Branded => &COLOR_BRANDED_PRIMARY,
        }
    }

    pub fn color_hover_class(&self) -> &'static str {
        match self {
            Self::Regular => &COLOR_REGULAR,
            Self::Branded => &COLOR_BRANDED_PRIMARY,
        }
    }

    pub fn bg_disabled_class(self) -> &'static str {
        match self {
            Self::Regular => &BG_COLOR_REGULAR_DISABLED,
            Self::Branded => &BG_COLOR_BRANDED_PRIMARY_DISABLED,
        }
    }

    pub fn color_disabled_class(self) -> &'static str {
        match self {
            Self::Regular => &COLOR_REGULAR_DISABLED,
            Self::Branded => &COLOR_BRANDED_PRIMARY_DISABLED,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ButtonStyle {
    Solid,
}

pub struct Button {
    size: ButtonSize,
    style: ButtonStyle,
    color: ButtonColor,
    text: String,
    disabled_signal: Option<Pin<Box<dyn Signal<Item = bool>>>>,
    on_click: Option<Box<dyn FnMut()>>,
    mixin: Option<Box<dyn MixinFnOnce<HtmlElement>>>,
}

impl Button {
    pub fn new() -> Self {
        Self {
            size: ButtonSize::Lg,
            style: ButtonStyle::Solid,
            color: ButtonColor::Branded,
            text: "".to_string(),
            disabled_signal: None,
            on_click: None,
            mixin: None,
        }
    }

    pub fn with_text(mut self, text: impl ToString) -> Self {
        self.text = text.to_string();
        self
    }

    pub fn with_style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn with_color(mut self, color: ButtonColor) -> Self {
        self.color = color;
        self
    }

    pub fn with_disabled_signal(
        mut self,
        disabled_signal: impl Signal<Item = bool> + 'static,
    ) -> Self {
        self.disabled_signal = Some(Box::pin(disabled_signal));
        self
    }

    pub fn with_on_click(mut self, on_click: impl FnMut() + 'static) -> Self {
        self.on_click = Some(Box::new(on_click));
        self
    }

    pub fn with_mixin(mut self, mixin: impl MixinFnOnce<HtmlElement> + 'static) -> Self {
        self.mixin = Some(Box::new(mixin));
        self
    }

    pub fn render(self) -> Dom {
        static CLASS: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "inline-flex")
                .style("justify-content", "center")
                .style("align-items", "center")
                .style("gap", "0.625rem")
                .style("border-radius", "0.25rem")
                .style("width", "fit-content")
            }
        });

        let Self {
            size,
            color,
            text,
            disabled_signal: disabled_sig,
            mut on_click,
            style,
            mixin,
        } = self;

        let hovering = Mutable::new(false);

        // doing this instead of a Broadcaster because we want to:
        // 1. prevent the on_click handler being called if disabled signal is true
        // 2. show cursor style of not-allowed if disabled signal is true (so setting pointer-events: none doesn't work here)
        let disabled = Mutable::new(false);

        let neither_hover_nor_disabled_signal = || {
            map_ref! {
                let disabled = disabled.signal(),
                let hovering = hovering.signal() => {
                    !*disabled && !*hovering
                }
            }
        };

        let hover_but_not_disabled_signal = || {
            map_ref! {
                let disabled = disabled.signal(),
                let hovering = hovering.signal() => {
                    !*disabled && *hovering
                }
            }
        };

        let cursor_signal = map_ref! {
            let disabled = disabled.signal(),
            let hovering = hovering.signal() => {
                if *disabled {
                    "not-allowed"
                } else if *hovering {
                    "pointer"
                } else {
                    "auto"
                }
            }
        };

        html!("div", {
            .apply_if(disabled_sig.is_some(), clone!(disabled => move |dom| {
                dom
                    .future(disabled_sig.unwrap_ext().for_each(clone!(disabled => move |is_disabled| {
                        clone!(disabled => async move {
                            disabled.set_neq(is_disabled);
                        })
                    })))
            }))
            .class([&*USER_SELECT_NONE, &*CLASS, size.container_class(), size.text_size_class()])
            .apply(set_on_hover(&hovering))
            .style_signal("cursor", cursor_signal)
            .class_signal(color.bg_disabled_class(), disabled.signal())
            .class_signal(color.bg_hover_class(), hover_but_not_disabled_signal())
            .class_signal(color.bg_class(), neither_hover_nor_disabled_signal())
            .apply(handle_on_click(clone!(disabled => move || {
                if !disabled.get() {
                    if let Some(on_click) = &mut on_click {
                        on_click();
                    }
                }
            })))
            .apply_if(mixin.is_some(), |dom| {
                mixin.unwrap_ext()(dom)
            })
            .children([
                html!("div", {
                    .class_signal(color.color_disabled_class(), disabled.signal())
                    .class_signal(color.color_hover_class(), hover_but_not_disabled_signal())
                    .class_signal(color.color_class(), neither_hover_nor_disabled_signal())
                    .text(&text)
                }),
            ])
        })
    }
}
