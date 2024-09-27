#![allow(async_fn_in_trait)]

pub mod http;
pub mod io;

mod runtime;

pub use runtime::block_on;
use runtime::Reactor;

use std::sync::LazyLock;

static REACTOR: LazyLock<Reactor> = LazyLock::new(Reactor::new);

pub fn reactor() -> &'static Reactor {
    LazyLock::force(&REACTOR)
}
