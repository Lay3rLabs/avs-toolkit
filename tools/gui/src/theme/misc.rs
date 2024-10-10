use std::sync::LazyLock;

use dominator::class;

pub static USER_SELECT_NONE: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style(["-moz-user-select", "user-select"], "none")
    }
});

pub static POINTER_EVENTS_NONE: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("pointer-events", "none")
    }
});

pub static CURSOR_POINTER: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("cursor", "pointer")
    }
});

pub static WORD_WRAP_PRE: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("white-space", "pre-wrap")
    }
});

pub static TEXT_ALIGN_CENTER: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("text-align", "center")
    }
});
