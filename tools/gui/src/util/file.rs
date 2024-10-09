use crate::prelude::*;
use avs_toolkit_shared::file::WasmFile;
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys, File};

pub async fn read_file_bytes(file: &File) -> Result<Vec<u8>> {
    let array_buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(|e| anyhow!("{e:?}"))?;
    Ok(js_sys::Uint8Array::new(&array_buffer).to_vec())
}
