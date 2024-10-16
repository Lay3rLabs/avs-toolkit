use crate::prelude::*;

pub static FILTERS_DP_2: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("box-shadow", "0px 1px 8px rgba(6, 9, 11, 0.04), 0px 2px 4px rgba(0, 0, 0, 0.08), 0px 1px 2px rgba(0, 0, 0, 0.12)")
    }
});

pub static FILTERS_DP_4: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("box-shadow", "0px 1px 12px rgba(6, 9, 11, 0.08), 0px 4px 8px rgba(0, 0, 0, 0.12), 0px 2px 4px rgba(0, 0, 0, 0.16)")
    }
});

pub static FILTERS_DP_6: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("box-shadow", "0px 1px 16px rgba(6, 9, 11, 0.12), 0px 8px 12px rgba(0, 0, 0, 0.16), 0px 4px 8px rgba(0, 0, 0, 0.2)")
    }
});

pub static FILTERS_DP_8: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("box-shadow", "0px 1px 20px rgba(0, 0, 0, 0.16), 0px 12px 16px rgba(6, 9, 11, 0.2), 0px 6px 12px rgba(0, 0, 0, 0.24),inset 0px 0.5px 1px rgba(255, 255, 255, 0.15)")
    }
});
