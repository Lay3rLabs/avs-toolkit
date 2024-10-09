use wasmtime::component::bindgen;
bindgen!({
  path: "../../wit",
  world: "task-queue",
  async: true,
  with: {
      "wasi": wasmtime_wasi::bindings,
      "wasi:http@0.2.0": wasmtime_wasi_http::bindings::http,
  },
});
