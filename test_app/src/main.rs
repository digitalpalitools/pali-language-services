use pls_core::inflections::PlsInflectionsHost;
use rusqlite::{Connection, Row, NO_PARAMS};

fn get_row_cells(row: &Row) -> Vec<String> {
    let cells: Vec<String> = row
        .column_names()
        .iter()
        .map(|&cn| {
            let cell: String = match row.get(cn) {
                Ok(c) => c,
                Err(e) => e.to_string(),
            };
            cell
        })
        .collect();

    cells
}

fn exec_sql_core(sql: &str) -> rusqlite::Result<Vec<Vec<Vec<String>>>, rusqlite::Error> {
    let conn = Connection::open("../inflections.db")?;
    let mut result: Vec<Vec<Vec<String>>> = Vec::new();
    for s in sql.split(';').filter(|s| !s.trim().is_empty()) {
        let mut stmt = conn.prepare(&s)?;
        let mut rows = stmt.query(NO_PARAMS)?;

        let mut table: Vec<Vec<String>> = Vec::new();
        while let Some(row) = rows.next()? {
            table.push(get_row_cells(row));
        }
        result.push(table)
    }

    Ok(result)
}

struct Host {}

impl<'a> PlsInflectionsHost<'a> for Host {
    fn get_locale(&self) -> &'a str {
        "en"
    }

    fn get_version(&self) -> &'a str {
        "host version v0.0.1"
    }

    fn get_url(&self) -> &'a str {
        "the table is hosted here"
    }

    fn transliterate(&self, s: &str) -> Result<String, String> {
        Ok(s.to_string())
    }

    fn exec_sql_query_core(&self, sql: &str) -> Result<String, String> {
        let table = exec_sql_core(&sql).map_err(|x| x.to_string())?;
        serde_json::to_string(&table).map_err(|x| x.to_string())
    }
}

fn main() -> Result<(), String> {
    println!("{:?}", pls_core::alphabet::PALI_ALPHABET_ROMAN);
    let x = pls_core::alphabet::PaliAlphabet::Aa;
    println!("ā > bh? {:#?}", x > pls_core::alphabet::PaliAlphabet::Bh);

    let html = pls_core::inflections::generate_inflection_table("kāmaṃ 3", &Host {})?;
    println!("{:#?}", html);

    Ok(())
}
