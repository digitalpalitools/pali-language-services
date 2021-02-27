use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn string_compare(str1: &str, str2: &str) -> isize {
    pls_core::alphabet::string_compare(str1, str2)
}

#[wasm_bindgen]
pub fn string_length(str1: &str) -> usize {
    pls_core::alphabet::string_length(str1)
}
