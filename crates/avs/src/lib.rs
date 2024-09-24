#![allow(async_fn_in_trait)]

mod runtime;
mod task_queue;

pub mod http;
pub mod io;
pub mod iter;
pub mod rand;
pub mod time;

pub use runtime::block_on;
use runtime::Reactor;

use std::sync::LazyLock;

static REACTOR: LazyLock<Reactor> = LazyLock::new(Reactor::new);

pub fn reactor() -> &'static Reactor {
    LazyLock::force(&REACTOR)
}
