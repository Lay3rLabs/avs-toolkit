use std::sync::LazyLock;

use dominator::{class, styles};
use futures_signals::signal::SignalExt;

pub const FONT_FAMILY_ROBOTO: &str = r#""Roboto", sans-serif"#;
pub const FONT_FAMILY_INTER: &str = r#""Inter", sans-serif"#;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontSize {
    Hero,
    Header,
    Title,
    Primary,
    Body,
    Button,
    ButtonSmall,
    Link,
    Secondary,
    Caption,
}

impl FontSize {
    pub fn class(self) -> &'static str {
        // these could all individually have .style_signal
        // driven from Breakpoint::signal()
        // but instead we just set the font-size directly
        // on the root element and rems flow from there
        static HERO: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("font-size", "1.62rem")
            }
        });

        static HEADER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("font-size", "1.25rem")
            }
        });

        static TITLE: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("font-size", "1rem")
            }
        });

        static PRIMARY: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("font-size", "0.94rem")
            }
        });

        static BODY: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("font-size", "0.88rem")
            }
        });

        static BUTTON: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("font-size", "0.88rem")
            }
        });

        static BUTTON_SMALL: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("font-size", "0.75rem")
            }
        });

        static LINK: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("font-size", "0.81rem")
            }
        });

        static SECONDARY: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("font-size", "0.81rem")
            }
        });

        static CAPTION: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("font-size", "0.75rem")
            }
        });

        match self {
            Self::Hero => &*HERO,
            Self::Header => &*HEADER,
            Self::Title => &*TITLE,
            Self::Primary => &*PRIMARY,
            Self::Body => &*BODY,
            Self::Button => &*BUTTON,
            Self::ButtonSmall => &*BUTTON_SMALL,
            Self::Link => &*LINK,
            Self::Secondary => &*SECONDARY,
            Self::Caption => &*CAPTION,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    SemiBold,
    Bold,
}

impl FontWeight {
    pub fn class(self) -> &'static str {
        static SEMI_BOLD: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("font-weight", "600")
            }
        });

        static BOLD: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("font-weight", "700")
            }
        });

        match self {
            Self::SemiBold => &*SEMI_BOLD,
            Self::Bold => &*BOLD,
        }
    }
}
