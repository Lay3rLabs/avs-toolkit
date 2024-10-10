use std::sync::LazyLock;

use dominator::{class, styles};
use futures_signals::signal::SignalExt;

pub const FONT_FAMILY_ROBOTO: &str = r#""Roboto", sans-serif"#;

pub static TEXT_SIZE_H1: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("font-size", "5.125rem")
    }
});

pub static TEXT_SIZE_H2: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("font-size", "3.1875rem")
        .style("line-height", "3.825rem")
    }
});

pub static TEXT_SIZE_H3: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("font-size", "1.9375rem")
    }
});

pub static TEXT_SIZE_XLG: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("font-size", "1.5rem")
    }
});

pub static TEXT_SIZE_LG: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("font-size", "1.1875rem")
    }
});

pub static TEXT_SIZE_MD: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("font-size", "0.875rem")
    }
});

pub static TEXT_SIZE_SM: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("font-size", "0.75rem")
    }
});

pub static TEXT_WEIGHT_BOLD: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("font-weight", "700")
    }
});
