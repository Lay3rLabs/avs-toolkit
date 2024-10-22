use crate::prelude::*;

pub static COLOR_TEXT_INTERACTIVE_ERROR: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("color", ColorTextInteractive::Error.signal())
    }
});

impl ColorText {
    pub fn color_class(self) -> &'static str {
        static BRAND: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style_signal("color", ColorText::Brand.signal())
            }
        });

        static PRIMARY: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style_signal("color", ColorText::Primary.signal())
            }
        });

        static BODY: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style_signal("color", ColorText::Body.signal())
            }
        });

        static SECONDARY: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style_signal("color", ColorText::Secondary.signal())
            }
        });

        static TERTIARY: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style_signal("color", ColorText::Tertiary.signal())
            }
        });

        match self {
            ColorText::Primary => &*PRIMARY,
            ColorText::Body => &*BODY,
            ColorText::Secondary => &*SECONDARY,
            ColorText::Tertiary => &*TERTIARY,
            ColorText::Brand => &*BRAND,
        }
    }
}

pub static BG_COLOR_INTERACTIVE_DEFAULT: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("background-color", ColorBackgroundInteractive::Default.signal())
    }
});

pub static BG_COLOR_INTERACTIVE_HOVER: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("background-color", ColorBackgroundInteractive::Hover.signal())
    }
});

pub static BG_COLOR_INTERACTIVE_SELECTED: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("background-color", ColorBackgroundInteractive::Selected.signal())
    }
});

pub static BG_COLOR_INTERACTIVE_PRESSED: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("background-color", ColorBackgroundInteractive::Pressed.signal())
    }
});

pub static BG_COLOR_INTERACTIVE_DISABLED: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("background-color", ColorBackgroundInteractive::Disabled.signal())
    }
});

pub static BG_COLOR_INTERACTIVE_ACTIVE: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("background-color", ColorBackgroundInteractive::Active.signal())
    }
});

pub static BG_COLOR_INTERACTIVE_ERROR: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("background-color", ColorBackgroundInteractive::Error.signal())
    }
});

pub static BG_COLOR_INTERACTIVE_WARNING: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("background-color", ColorBackgroundInteractive::Warning.signal())
    }
});

pub static BG_COLOR_INTERACTIVE_VALID: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style_signal("background-color", ColorBackgroundInteractive::Valid.signal())
    }
});
