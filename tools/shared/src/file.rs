pub enum WasmFile {
    Url(String),
    Bytes(Vec<u8>),
}
