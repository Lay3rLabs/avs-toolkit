#![allow(warnings)]
mod atoms;
mod client;
mod config;
mod page;
mod prelude;
mod route;
mod theme;
mod util;

use crate::prelude::*;

#[wasm_bindgen(start)]
pub async fn run() -> Result<(), JsValue> {
    init_logger();
    std::panic::set_hook(Box::new(on_panic));

    theme::stylesheet::init();

    dominator::append_dom(&dominator::body(), route::render());

    Ok(())
}

cfg_if::cfg_if! {
    if #[cfg(all(feature = "wasm-logger", feature = "console_error_panic_hook"))] {
        fn init_logger() {
            wasm_logger::init(wasm_logger::Config::default());
            console_error_panic_hook::set_once();
            log::info!("rust logging enabled!!!");
        }
    } else {
        fn init_logger() {
            log::info!("rust logging disabled!"); //<-- won't be seen
        }
    }
}

fn on_panic(info: &std::panic::PanicInfo) {
    log::error!("panic: {:?}", info);
    web_sys::window()
        .unwrap()
        .alert_with_message("got a panic!")
        .unwrap();
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::hook(info);
}
