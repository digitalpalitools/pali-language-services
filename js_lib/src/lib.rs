use pls_core::inflections::InflectionsHost;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(raw_module = "./pali_language_services_dal.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = transliterate)]
    fn transliterate(sql: &str) -> Result<String, JsValue>;

    #[wasm_bindgen(catch, js_name = execSql)]
    fn exec_sql(sql: &str) -> Result<String, JsValue>;
}

#[wasm_bindgen(js_name = stringCompare)]
pub fn string_compare(str1: &str, str2: &str) -> isize {
    pls_core::alphabet::string_compare(str1, str2)
}

#[wasm_bindgen(js_name = stringLength)]
pub fn string_length(str1: &str) -> usize {
    pls_core::alphabet::string_length(str1)
}

struct Host<'a> {
    locale: &'a str,
    version: &'a str,
    url: &'a str,
}

impl<'a> InflectionsHost<'a> for Host<'a> {
    fn get_locale(&self) -> &'a str {
        self.locale
    }

    fn get_version(&self) -> &'a str {
        self.version
    }

    fn get_url(&self) -> &'a str {
        self.url
    }

    fn transliterate(&self, s: &str) -> Result<String, String> {
        transliterate(s).map_err(|e| {
            e.as_string()
                .unwrap_or_else(|| "No exception string!".to_string())
        })
    }

    fn exec_sql_query_core(&self, sql: &str) -> Result<String, String> {
        exec_sql(sql).map_err(|e| {
            e.as_string()
                .unwrap_or_else(|| "No exception string!".to_string())
        })
    }
}

#[wasm_bindgen(js_name = generateInflectionTable)]
pub fn generate_inflection_table(
    pali1: &str,
    host_url: &str,
    host_version: &str,
    locale: &str,
) -> String {
    pls_core::inflections::generate_inflection_table(
        pali1,
        &Host {
            locale,
            url: host_url,
            version: host_version,
        },
    )
    .unwrap()
}
