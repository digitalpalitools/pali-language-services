use wasm_bindgen::prelude::*;

#[wasm_bindgen(raw_module = "./pali_language_services_dal.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = execSql)]
    fn exec_sql(sql: String) -> Result<String, JsValue>;

    #[wasm_bindgen(catch, js_name = execSqlWithTransliteration)]
    fn exec_sql_with_transliteration(sql: String) -> Result<String, JsValue>;
}

#[wasm_bindgen(js_name = stringCompare)]
pub fn string_compare(str1: &str, str2: &str) -> isize {
    pls_core::alphabet::string_compare(str1, str2)
}

#[wasm_bindgen(js_name = stringLength)]
pub fn string_length(str1: &str) -> usize {
    pls_core::alphabet::string_length(str1)
}

#[wasm_bindgen(js_name = generateInflectionTable)]
pub fn generate_inflection_table(pali1: &str) -> String {
    pls_core::inflections::generate_inflection_table(
        pali1,
        |s| {
            exec_sql(s).map_err(|e| {
                e.as_string()
                    .unwrap_or_else(|| "No exception string!".to_string())
            })
        },
        |s| {
            exec_sql_with_transliteration(s).map_err(|e| {
                e.as_string()
                    .unwrap_or_else(|| "No exception string!".to_string())
            })
        },
    )
    .unwrap()
}
