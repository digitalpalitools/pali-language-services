use crate::logger::PlsLogger;
use crate::sql_access::SqlAccess;
use pls_core::inflections::host::PlsInflectionsHost;

pub struct PlsHost<'a> {
    pub locale: &'a str,
    pub version: &'a str,
    pub url: &'a str,
    pub sql_access: SqlAccess,
    pub logger: &'a dyn PlsLogger,
}

impl<'a> PlsInflectionsHost<'a> for PlsHost<'a> {
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
        Ok(s.to_string())
    }

    fn exec_sql_query_core(&self, sql: &str) -> Result<String, String> {
        let table = self.sql_access.exec(&sql)?;
        serde_json::to_string(&table).map_err(|x| x.to_string())
    }

    fn log_warning(&self, msg: &str) {
        self.logger.warning(msg)
    }
}
