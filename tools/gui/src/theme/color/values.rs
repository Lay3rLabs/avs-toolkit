use dominator::ColorScheme;

use crate::prelude::*;

use super::scheme::color_scheme_signal;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorText {
    Primary,
    Body,
    Secondary,
    Tertiary,
    Brand,
}

impl ColorText {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Primary => match color {
                ColorScheme::Light => "#151617",
                ColorScheme::Dark => "#ffffff",
            },
            Self::Body => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.9500)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.9500)",
            },
            Self::Secondary => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.7000)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.7000)",
            },
            Self::Tertiary => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.5000)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.5000)",
            },
            Self::Brand => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.9500)",
                ColorScheme::Dark => "rgba(155, 110, 240, 0.9500)",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorTextInteractive {
    Disabled,
    Active,
    Error,
    Warning,
    Valid,
    BrandDisabled,
}

impl ColorTextInteractive {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Disabled => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.2000)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.2000)",
            },
            Self::Active => match color {
                ColorScheme::Light => "rgba(196, 160, 255, 0.9500)",
                ColorScheme::Dark => "rgba(196, 160, 255, 0.9500)",
            },
            Self::Error => match color {
                ColorScheme::Light => "rgba(199, 62, 89, 0.9500)",
                ColorScheme::Dark => "rgba(199, 62, 89, 0.9500)",
            },
            Self::Warning => match color {
                ColorScheme::Light => "rgba(215, 147, 73, 0.9500)",
                ColorScheme::Dark => "rgba(215, 147, 73, 0.9500)",
            },
            Self::Valid => match color {
                ColorScheme::Light => "rgba(57, 166, 153, 0.9500)",
                ColorScheme::Dark => "rgba(57, 166, 153, 0.9500)",
            },
            Self::BrandDisabled => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.2500)",
                ColorScheme::Dark => "rgba(196, 160, 255, 0.3500)",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorTextInteractiveButton {
    Primary,
    Disabled,
}

impl ColorTextInteractiveButton {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Primary => match color {
                ColorScheme::Light => "rgba(255, 255, 255, 0.9500)",
                ColorScheme::Dark => "rgba(21, 22, 23, 0.9500)",
            },
            Self::Disabled => match color {
                ColorScheme::Light => "rgba(243, 246, 248, 0.7500)",
                ColorScheme::Dark => "rgba(36, 38, 40, 0.7500)",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorIcon {
    Primary,
    Secondary,
    Tertiary,
    Brand,
}

impl ColorIcon {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Primary => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.9000)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.9000)",
            },
            Self::Secondary => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.6000)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.6000)",
            },
            Self::Tertiary => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.4000)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.4000)",
            },
            Self::Brand => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.9000)",
                ColorScheme::Dark => "rgba(155, 110, 240, 0.9000)",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorIconInteractive {
    Disabled,
    Active,
    Error,
    Warning,
    Valid,
    BrandDisabled,
}

impl ColorIconInteractive {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Disabled => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.1500)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.1500)",
            },
            Self::Active => match color {
                ColorScheme::Light => "rgba(196, 160, 255, 0.9000)",
                ColorScheme::Dark => "rgba(196, 160, 255, 0.9000)",
            },
            Self::Error => match color {
                ColorScheme::Light => "rgba(199, 62, 89, 0.9000)",
                ColorScheme::Dark => "rgba(199, 62, 89, 0.9000)",
            },
            Self::Warning => match color {
                ColorScheme::Light => "rgba(215, 147, 73, 0.9000)",
                ColorScheme::Dark => "rgba(215, 147, 73, 0.9000)",
            },
            Self::Valid => match color {
                ColorScheme::Light => "rgba(57, 166, 153, 0.9000)",
                ColorScheme::Dark => "rgba(57, 166, 153, 0.9000)",
            },
            Self::BrandDisabled => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.2000)",
                ColorScheme::Dark => "rgba(155, 110, 240, 0.3000)",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorIconInteractiveButton {
    Primary,
    Disabled,
}

impl ColorIconInteractiveButton {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Primary => match color {
                ColorScheme::Light => "rgba(243, 246, 248, 0.9500)",
                ColorScheme::Dark => "rgba(36, 38, 40, 0.9000)",
            },
            Self::Disabled => match color {
                ColorScheme::Light => "rgba(243, 246, 248, 0.7000)",
                ColorScheme::Dark => "rgba(36, 38, 40, 0.7000)",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorBorder {
    Base,
    Primary,
    Secondary,
}

impl ColorBorder {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Base => match color {
                ColorScheme::Light => "rgba(0, 0, 0, 0.0500)",
                ColorScheme::Dark => "#000000",
            },
            Self::Primary => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.1000)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.1500)",
            },
            Self::Secondary => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.0500)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.0500)",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorBorderInteractive {
    Hover,
    Selected,
    Focus,
    Disabled,
    Active,
    Error,
    Warning,
    Valid,
    BrandDisabled,
}

impl ColorBorderInteractive {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Hover => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.1500)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.1500)",
            },
            Self::Selected => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.2000)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.2000)",
            },
            Self::Focus => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.2000)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.2000)",
            },
            Self::Disabled => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.0500)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.0500)",
            },
            Self::Active => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.6500)",
                ColorScheme::Dark => "rgba(155, 110, 240, 0.6500)",
            },
            Self::Error => match color {
                ColorScheme::Light => "rgba(199, 62, 89, 0.6500)",
                ColorScheme::Dark => "rgba(199, 62, 89, 0.6500)",
            },
            Self::Warning => match color {
                ColorScheme::Light => "rgba(215, 147, 73, 0.6500)",
                ColorScheme::Dark => "rgba(215, 147, 73, 0.6500)",
            },
            Self::Valid => match color {
                ColorScheme::Light => "rgba(57, 166, 153, 0.6500)",
                ColorScheme::Dark => "rgba(57, 166, 153, 0.6500)",
            },
            Self::BrandDisabled => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.1500)",
                ColorScheme::Dark => "rgba(155, 110, 240, 0.2500)",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorBackground {
    Primary,
    Secondary,
    Tertiary,
    Button,
    Base,
    Card,
    Overlay,
    Inverse,
    Inverted,
}

impl ColorBackground {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Primary => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.0800)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.0800)",
            },
            Self::Secondary => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.0500)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.0500)",
            },
            Self::Tertiary => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.0300)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.0300)",
            },
            Self::Button => match color {
                ColorScheme::Light => "rgba(21, 22, 23, 0.9000)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.9000)",
            },
            Self::Base => match color {
                ColorScheme::Light => "#fbfaf9",
                ColorScheme::Dark => "#1c1a1e",
            },
            Self::Card => match color {
                ColorScheme::Light => "#ffffff",
                ColorScheme::Dark => "#151617",
            },
            Self::Overlay => match color {
                ColorScheme::Light => "rgba(0, 0, 0, 0.2000)",
                ColorScheme::Dark => "rgba(0, 0, 0, 0.7000)",
            },
            Self::Inverse => match color {
                ColorScheme::Light => "rgba(255, 255, 255, 0.6000)",
                ColorScheme::Dark => "rgba(21, 22, 23, 0.1500)",
            },
            Self::Inverted => match color {
                ColorScheme::Light => "#151617",
                ColorScheme::Dark => "#f3f6f8",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorBackgroundInteractive {
    Default,
    Hover,
    Selected,
    Pressed,
    Disabled,
    Active,
    Error,
    Warning,
    Valid,
}

impl ColorBackgroundInteractive {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Default => match color {
                ColorScheme::Light => "rgba(21, 22, 23, 0.0000)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.0000)",
            },
            Self::Hover => match color {
                ColorScheme::Light => "rgba(21, 22, 23, 0.1000)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.1000)",
            },
            Self::Selected => match color {
                ColorScheme::Light => "rgba(21, 22, 23, 0.1500)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.1500)",
            },
            Self::Pressed => match color {
                ColorScheme::Light => "rgba(21, 22, 23, 0.1500)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.1500)",
            },
            Self::Disabled => match color {
                ColorScheme::Light => "rgba(21, 22, 23, 0.0300)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.0300)",
            },
            Self::Active => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.2500)",
                ColorScheme::Dark => "rgba(155, 110, 240, 0.2500)",
            },
            Self::Error => match color {
                ColorScheme::Light => "rgba(199, 62, 89, 0.2000)",
                ColorScheme::Dark => "rgba(199, 62, 89, 0.1500)",
            },
            Self::Warning => match color {
                ColorScheme::Light => "rgba(215, 147, 73, 0.2000)",
                ColorScheme::Dark => "rgba(215, 147, 73, 0.1500)",
            },
            Self::Valid => match color {
                ColorScheme::Light => "rgba(57, 166, 153, 0.1000)",
                ColorScheme::Dark => "rgba(57, 166, 153, 0.1000)",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorBackgroundButton {
    Hover,
    Pressed,
    Progress,
    Disabled,
    Active,
}

impl ColorBackgroundButton {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Hover => match color {
                ColorScheme::Light => "rgba(21, 22, 23, 0.9500)",
                ColorScheme::Dark => "rgba(255, 255, 255, 0.9500)",
            },
            Self::Pressed => match color {
                ColorScheme::Light => "#000000",
                ColorScheme::Dark => "#ffffff",
            },
            Self::Progress => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.7500)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.7500)",
            },
            Self::Disabled => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.4000)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.4000)",
            },
            Self::Active => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.9000)",
                ColorScheme::Dark => "rgba(155, 110, 240, 0.9000)",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorComponent {
    Modal,
    Dropdown,
    Tooltip,
    Toast,
    Widget,
    ListElement,
}

impl ColorComponent {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Modal => match color {
                ColorScheme::Light => "#ffffff",
                ColorScheme::Dark => "#151617",
            },
            Self::Dropdown => match color {
                ColorScheme::Light => "#ffffff",
                ColorScheme::Dark => "#1e1f20",
            },
            Self::Tooltip => match color {
                ColorScheme::Light => "#0f0f10",
                ColorScheme::Dark => "#0f0f10",
            },
            Self::Toast => match color {
                ColorScheme::Light => "#0c0c0d",
                ColorScheme::Dark => "#0c0c0d",
            },
            Self::Widget => match color {
                ColorScheme::Light => "rgba(243, 246, 248, 0.4000)",
                ColorScheme::Dark => "rgba(21, 22, 23, 0.2000)",
            },
            Self::ListElement => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.0200)",
                ColorScheme::Dark => "rgba(21, 22, 23, 0.0200)",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorComponentBadge {
    Primary,
    Brand,
    Valid,
    Warning,
    Error,
}

impl ColorComponentBadge {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Primary => match color {
                ColorScheme::Light => "rgba(36, 38, 40, 0.2500)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.2000)",
            },
            Self::Brand => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.4000)",
                ColorScheme::Dark => "rgba(155, 110, 240, 0.4000)",
            },
            Self::Valid => match color {
                ColorScheme::Light => "rgba(57, 166, 153, 0.4500)",
                ColorScheme::Dark => "rgba(57, 166, 153, 0.4500)",
            },
            Self::Warning => match color {
                ColorScheme::Light => "rgba(215, 147, 73, 0.4500)",
                ColorScheme::Dark => "rgba(215, 147, 73, 0.4500)",
            },
            Self::Error => match color {
                ColorScheme::Light => "rgba(199, 62, 89, 0.4500)",
                ColorScheme::Dark => "rgba(199, 62, 89, 0.4500)",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorComponentCode {
    Background,
    Green,
    Red,
}

impl ColorComponentCode {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Background => match color {
                ColorScheme::Light => "#ededeb",
                ColorScheme::Dark => "#111219",
            },
            Self::Green => match color {
                ColorScheme::Light => "rgba(182, 214, 128, 0.9500)",
                ColorScheme::Dark => "rgba(182, 214, 128, 0.9500)",
            },
            Self::Red => match color {
                ColorScheme::Light => "rgba(218, 97, 92, 0.9500)",
                ColorScheme::Dark => "rgba(218, 97, 92, 0.9500)",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorBranded {
    Primary,
    Secondary,
}

impl ColorBranded {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Primary => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.9000)",
                ColorScheme::Dark => "rgba(155, 110, 240, 0.9000)",
            },
            Self::Secondary => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.1500)",
                ColorScheme::Dark => "rgba(155, 110, 240, 0.1500)",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorBrandedPrimary {
    Hover,
    Presed,
    Disabled,
    Text,
    Icon,
    TextDisabled,
    IconDisabled,
}

impl ColorBrandedPrimary {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Hover => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.9500)",
                ColorScheme::Dark => "rgba(155, 110, 240, 0.9500)",
            },
            Self::Presed => match color {
                ColorScheme::Light => "#9b6ef0",
                ColorScheme::Dark => "#9b6ef0",
            },
            Self::Disabled => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.3000)",
                ColorScheme::Dark => "rgba(155, 110, 240, 0.3000)",
            },
            Self::Text => match color {
                ColorScheme::Light => "rgba(255, 255, 255, 0.9500)",
                ColorScheme::Dark => "rgba(255, 255, 255, 0.9500)",
            },
            Self::Icon => match color {
                ColorScheme::Light => "rgba(243, 246, 248, 0.9000)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.9000)",
            },
            Self::TextDisabled => match color {
                ColorScheme::Light => "rgba(243, 246, 248, 0.9000)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.5000)",
            },
            Self::IconDisabled => match color {
                ColorScheme::Light => "rgba(243, 246, 248, 0.8500)",
                ColorScheme::Dark => "rgba(243, 246, 248, 0.4500)",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorBrandedSecondary {
    Hover,
    Presed,
    Disabled,
    Text,
    Icon,
    TextDisabled,
    IconDisabled,
}

impl ColorBrandedSecondary {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Hover => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.2000)",
                ColorScheme::Dark => "rgba(155, 110, 240, 0.2000)",
            },
            Self::Presed => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.2500)",
                ColorScheme::Dark => "rgba(155, 110, 240, 0.3000)",
            },
            Self::Disabled => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.0800)",
                ColorScheme::Dark => "rgba(155, 110, 240, 0.0800)",
            },
            Self::Text => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.9500)",
                ColorScheme::Dark => "rgba(196, 160, 255, 0.9500)",
            },
            Self::Icon => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.9000)",
                ColorScheme::Dark => "rgba(196, 160, 255, 0.9000)",
            },
            Self::TextDisabled => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.3500)",
                ColorScheme::Dark => "rgba(196, 160, 255, 0.4500)",
            },
            Self::IconDisabled => match color {
                ColorScheme::Light => "rgba(155, 110, 240, 0.3000)",
                ColorScheme::Dark => "rgba(196, 160, 255, 0.4000)",
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorAnnotation {
    Text,
    Icon,
}

impl ColorAnnotation {
    pub fn signal(self) -> impl Signal<Item = &'static str> {
        color_scheme_signal().map(move |color| match self {
            Self::Text => match color {
                ColorScheme::Light => "rgba(230, 73, 149, 0.7500)",
                ColorScheme::Dark => "rgba(230, 73, 149, 0.7500)",
            },
            Self::Icon => match color {
                ColorScheme::Light => "rgba(230, 73, 149, 0.5000)",
                ColorScheme::Dark => "rgba(230, 73, 149, 0.5000)",
            },
        })
    }
}
