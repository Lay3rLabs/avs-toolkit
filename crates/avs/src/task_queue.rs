#[doc(hidden)]
/// `wit-bindgen` generated code for internal use.
pub mod wit {
    #![allow(missing_docs)]

    wit_bindgen::generate!({
        world: "task-queue",
        path: "./wit",
        with: {
            "wasi:cli/environment@0.2.0": wasi::cli::environment,
            "wasi:clocks/monotonic-clock@0.2.0": wasi::clocks::monotonic_clock,
            "wasi:clocks/wall-clock@0.2.0": wasi::clocks::wall_clock,
            "wasi:filesystem/types@0.2.0": wasi::filesystem::types,
            "wasi:filesystem/preopens@0.2.0": wasi::filesystem::preopens,
            "wasi:http/types@0.2.0": wasi::http::types,
            "wasi:http/outgoing-handler@0.2.0": wasi::http::outgoing_handler,
            "wasi:io/error@0.2.0": wasi::io::error,
            "wasi:io/poll@0.2.0": wasi::io::poll,
            "wasi:io/streams@0.2.0": wasi::io::streams,
            "wasi:random/insecure-seed@0.2.0": wasi::random::insecure_seed,
            "wasi:random/insecure@0.2.0": wasi::random::insecure,
            "wasi:random/random@0.2.0": wasi::random::random,
        }
    });
}
