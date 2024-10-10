use std::sync::LazyLock;

use dominator::class;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    Darkest,
    Accent,
    AccentDarker,
    Whiteish,
    Darkish,
    Grey,
    GreyAlt1,
    GreyAlt2,
    Focus,
    Red,
    RedDarker,
    Orange,
    OrangeDarker,
    Green,
    GreenDarker,
    PureWhite,
}

impl Color {
    pub const fn hex_str(self) -> &'static str {
        match self {
            Self::Darkest => "#11131A",
            Self::Accent => "#28B9EA",
            Self::AccentDarker => "#1A8ABF",
            Self::Whiteish => "#FAFAFA",
            Self::Darkish => "#45474F",
            Self::Grey => "#92949F",
            Self::GreyAlt1 => "#D9D9D9",
            Self::GreyAlt2 => "#EFEFEF",
            Self::Focus => "#73A2FF",
            Self::Red => "#E00C0C",
            Self::RedDarker => "#B80C0C",
            Self::Orange => "#ED933F",
            Self::OrangeDarker => "#D17F3A",
            Self::Green => "#3AD365",
            Self::GreenDarker => "#2E9F4A",
            Self::PureWhite => "#FFFFFF",
        }
    }

    pub fn class(&self) -> &str {
        pub static DARKEST: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", Color::Darkest.hex_str())
            }
        });

        pub static ACCENT: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", Color::Accent.hex_str())
            }
        });

        pub static ACCENT_DARKER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", Color::AccentDarker.hex_str())
            }
        });

        pub static WHITEISH: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", Color::Whiteish.hex_str())
            }
        });

        pub static DARKISH: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", Color::Darkish.hex_str())
            }
        });

        pub static GREY: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", Color::Grey.hex_str())
            }
        });

        pub static GREY_ALT1: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", Color::GreyAlt1.hex_str())
            }
        });

        pub static GREY_ALT2: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", Color::GreyAlt2.hex_str())
            }
        });

        pub static FOCUS: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", Color::Focus.hex_str())
            }
        });

        pub static RED: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", Color::Red.hex_str())
            }
        });

        pub static RED_DARKER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", Color::RedDarker.hex_str())
            }
        });

        pub static ORANGE: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", Color::Orange.hex_str())
            }
        });

        pub static ORANGE_DARKER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", Color::OrangeDarker.hex_str())
            }
        });

        pub static GREEN: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", Color::Green.hex_str())
            }
        });

        pub static GREEN_DARKER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", Color::GreenDarker.hex_str())
            }
        });

        pub static PURE_WHITE: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("color", Color::PureWhite.hex_str())
            }
        });

        match self {
            Self::Darkest => &*DARKEST,
            Self::Accent => &*ACCENT,
            Self::AccentDarker => &*ACCENT_DARKER,
            Self::Whiteish => &*WHITEISH,
            Self::Darkish => &*DARKISH,
            Self::Grey => &*GREY,
            Self::GreyAlt1 => &*GREY_ALT1,
            Self::GreyAlt2 => &*GREY_ALT2,
            Self::Focus => &*FOCUS,
            Self::Red => &*RED,
            Self::RedDarker => &*RED_DARKER,
            Self::Orange => &*ORANGE,
            Self::OrangeDarker => &*ORANGE_DARKER,
            Self::Green => &*GREEN,
            Self::GreenDarker => &*GREEN_DARKER,
            Self::PureWhite => &*PURE_WHITE,
        }
    }

    pub fn class_bg(&self) -> &str {
        pub static DARKEST: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("background-color", Color::Darkest.hex_str())
            }
        });

        pub static ACCENT: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("background-color", Color::Accent.hex_str())
            }
        });

        pub static ACCENT_DARKER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("background-color", Color::AccentDarker.hex_str())
            }
        });

        pub static WHITEISH: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("background-color", Color::Whiteish.hex_str())
            }
        });

        pub static DARKISH: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("background-color", Color::Darkish.hex_str())
            }
        });

        pub static GREY: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("background-color", Color::Grey.hex_str())
            }
        });

        pub static GREY_ALT1: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("background-color", Color::GreyAlt1.hex_str())
            }
        });

        pub static GREY_ALT2: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("background-color", Color::GreyAlt2.hex_str())
            }
        });

        pub static FOCUS: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("background-color", Color::Focus.hex_str())
            }
        });

        pub static RED: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("background-color", Color::Red.hex_str())
            }
        });

        pub static RED_DARKER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("background-color", Color::RedDarker.hex_str())
            }
        });

        pub static ORANGE: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("background-color", Color::Orange.hex_str())
            }
        });

        pub static ORANGE_DARKER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("background-color", Color::OrangeDarker.hex_str())
            }
        });

        pub static GREEN: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("background-color", Color::Green.hex_str())
            }
        });

        pub static GREEN_DARKER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("background-color", Color::GreenDarker.hex_str())
            }
        });

        pub static PURE_WHITE: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("background-color", Color::PureWhite.hex_str())
            }
        });

        match self {
            Self::Darkest => &*DARKEST,
            Self::Accent => &*ACCENT,
            Self::AccentDarker => &*ACCENT_DARKER,
            Self::Whiteish => &*WHITEISH,
            Self::Darkish => &*DARKISH,
            Self::Grey => &*GREY,
            Self::GreyAlt1 => &*GREY_ALT1,
            Self::GreyAlt2 => &*GREY_ALT2,
            Self::Focus => &*FOCUS,
            Self::Red => &*RED,
            Self::RedDarker => &*RED_DARKER,
            Self::Orange => &*ORANGE,
            Self::OrangeDarker => &*ORANGE_DARKER,
            Self::Green => &*GREEN,
            Self::GreenDarker => &*GREEN_DARKER,
            Self::PureWhite => &*PURE_WHITE,
        }
    }

    pub fn class_border(&self) -> &str {
        pub static DARKEST: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("border-color", Color::Darkest.hex_str())
            }
        });

        pub static ACCENT: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("border-color", Color::Accent.hex_str())
            }
        });

        pub static ACCENT_DARKER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("border-color", Color::AccentDarker.hex_str())
            }
        });

        pub static WHITEISH: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("border-color", Color::Whiteish.hex_str())
            }
        });

        pub static DARKISH: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("border-color", Color::Darkish.hex_str())
            }
        });

        pub static GREY: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("border-color", Color::Grey.hex_str())
            }
        });

        pub static GREY_ALT1: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("border-color", Color::GreyAlt1.hex_str())
            }
        });

        pub static GREY_ALT2: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("border-color", Color::GreyAlt2.hex_str())
            }
        });

        pub static FOCUS: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("border-color", Color::Focus.hex_str())
            }
        });

        pub static RED: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("border-color", Color::Red.hex_str())
            }
        });

        pub static RED_DARKER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("border-color", Color::RedDarker.hex_str())
            }
        });

        pub static ORANGE: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("border-color", Color::Orange.hex_str())
            }
        });

        pub static ORANGE_DARKER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("border-color", Color::OrangeDarker.hex_str())
            }
        });

        pub static GREEN: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("border-color", Color::Green.hex_str())
            }
        });

        pub static GREEN_DARKER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("border-color", Color::GreenDarker.hex_str())
            }
        });

        pub static PURE_WHITE: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("border-color", Color::PureWhite.hex_str())
            }
        });

        match self {
            Self::Darkest => &*DARKEST,
            Self::Accent => &*ACCENT,
            Self::AccentDarker => &*ACCENT_DARKER,
            Self::Whiteish => &*WHITEISH,
            Self::Darkish => &*DARKISH,
            Self::Grey => &*GREY,
            Self::GreyAlt1 => &*GREY_ALT1,
            Self::GreyAlt2 => &*GREY_ALT2,
            Self::Focus => &*FOCUS,
            Self::Red => &*RED,
            Self::RedDarker => &*RED_DARKER,
            Self::Orange => &*ORANGE,
            Self::OrangeDarker => &*ORANGE_DARKER,
            Self::Green => &*GREEN,
            Self::GreenDarker => &*GREEN_DARKER,
            Self::PureWhite => &*PURE_WHITE,
        }
    }
}
