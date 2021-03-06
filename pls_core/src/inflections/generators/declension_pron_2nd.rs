pub fn create_html_body(
    table_name: &str,
    stem: &str,
    _transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(String) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<String, String> {
    let sql_v = exec_sql("select sqlite_version()".to_string())?;

    Ok(format!(
        "<div style='color: red'>{} (Declension Pron 2nd): () Not yet implemented! table name: {} - {}</div>",
        stem, table_name, sql_v[0][0][0]
    ))
}
