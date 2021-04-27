pub trait PlsInflectionsHost<'a> {
    fn get_locale(&self) -> &'a str;
    fn get_version(&self) -> &'a str;
    fn get_url(&self) -> &'a str;
    fn transliterate(&self, s: &str) -> Result<String, String>;
    fn exec_sql_query_core(&self, sql: &str) -> Result<String, String>;
    fn exec_sql_query(&self, sql: &str) -> Result<Vec<Vec<Vec<String>>>, String> {
        let result_str = self.exec_sql_query_core(sql)?;
        let result: Vec<Vec<Vec<String>>> =
            serde_json::from_str(&result_str).map_err(|e| e.to_string())?;
        Ok(result)
    }
    fn log_warning(&self, msg: &str);
}
