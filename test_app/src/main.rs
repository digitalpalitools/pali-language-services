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

fn exec_sql_core(sql: &str) -> rusqlite::Result<Vec<Vec<String>>, rusqlite::Error> {
    let conn = Connection::open("./inflections.db")?;
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut table: Vec<Vec<String>> = Vec::new();
    while let Some(row) = rows.next()? {
        table.push(get_row_cells(row));
    }

    Ok(table)
}

fn exec_sql(sql: String) -> Result<String, String> {
    let table = exec_sql_core(&sql).map_err(|x| x.to_string())?;
    serde_json::to_string(&table).map_err(|x| x.to_string())
}

fn exec_sql_with_transliteration(sql: String) -> Result<String, String> {
    let table = exec_sql_core(&sql).map_err(|x| x.to_string())?;
    serde_json::to_string(&table).map_err(|x| x.to_string())
}

fn main() -> Result<(), String> {
    println!("{:?}", pls_core::alphabet::PALI_ALPHABET_ROMAN);
    let x = pls_core::alphabet::PaliAlphabet::AA;
    println!("ā > bh? {:#?}", x > pls_core::alphabet::PaliAlphabet::BH);

    let html = pls_core::inflections::generate_inflection_table(
        "ābādheti",
        exec_sql,
        exec_sql_with_transliteration,
    )?;
    println!("{:#?}", html);

    std::fs::write("d:/delme/inflections.txt", &html).unwrap();

    Ok(())
}
