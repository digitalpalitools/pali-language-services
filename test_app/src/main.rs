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
    let conn = Connection::open("./inflections.db")?;
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

fn exec_sql(sql: &str) -> Result<String, String> {
    let table = exec_sql_core(&sql).map_err(|x| x.to_string())?;
    serde_json::to_string(&table).map_err(|x| x.to_string())
}

fn main() -> Result<(), String> {
    println!("{:?}", pls_core::alphabet::PALI_ALPHABET_ROMAN);
    let x = pls_core::alphabet::PaliAlphabet::AA;
    println!("ā > bh? {:#?}", x > pls_core::alphabet::PaliAlphabet::BH);

    let html = pls_core::inflections::generate_inflection_table(
        "vyābādhetvā",
        |s| Ok(s.to_string()),
        exec_sql,
    )?;
    println!("{:#?}", html);

    std::fs::write("d:/delme/inflections.txt", &html).unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // TODO: Need to be abstracted, granularized and moved to pls_core.
    fn basic_inflection_test() {
        let html = pls_core::inflections::generate_inflection_table(
            "ābādheti",
            |s| Ok(s.to_string()),
            exec_sql,
        );

        let approved_html = include_str!("test_data/basic_inflection_test.approved.txt");

        assert_eq!(html.unwrap(), approved_html);
    }
}
