use crate::prelude::*;
use gloo_events::EventListener;
use web_sys::MediaQueryListEvent;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Breakpoint {
    Phone,
    Tablet,
    Medium,
    Large,
}

impl Breakpoint {
    pub fn signal() -> impl Signal<Item = Self> {
        dominator::window_size().map(|size| {
            if size.width < 600.0 {
                Self::Phone
            } else if size.width < 900.0 {
                Self::Tablet
            } else if size.width < 1200.0 {
                Self::Medium
            } else {
                Self::Large
            }
        })
    }

    pub fn font_size(self) -> String {
        let pixels = match self {
            Self::Phone => 12.0,
            Self::Tablet => 14.0,
            Self::Medium => 16.0,
            Self::Large => 18.0,
        };

        format!("{}em", pixels / 16.0)
    }
}
