use rusqlite::{Connection, OptionalExtension, NO_PARAMS};

extern crate pali_language_services;

fn exec_isql(isql: &str) -> Result<String, String> {
    let c = Connection::open("d:/delme/inflections.db").map_err(|x| x.to_string())?;

    let results: Vec<String> = isql
        .split('|')
        .map(|sql| {
            let res = c
                .prepare(&sql)
                .map_err(|x| x.to_string())?
                .query_row(NO_PARAMS, |row| row.get(0))
                .optional()
                .map_err(|x| x.to_string())?
                .unwrap_or_else(|| "".to_string());
            Ok(res)
        })
        .map(|x| match x {
            Ok(x) => x,
            Err(e) => e,
        })
        .collect();

    Ok(results.join("|"))
}

fn get_itable(isql: &str) -> Result<String, String> {
    match isql.len() {
        0 => Err("?".to_string()),
        _ => Ok("eti_pr".to_string()),
    }
}

fn get_pali1_metadata(pali1: &str) -> Result<String, String> {
    match pali1.len() {
        0 => Err("?".to_string()),
        _ => Ok("ābādh|eti pr|like bhāveti".to_string()),
    }
}

fn main() {
    println!(
        "{:?}",
        pali_language_services::alphabet::PALI_ALPHABET_ROMAN
    );
    let x = pali_language_services::alphabet::PaliAlphabet::AA;
    println!(
        "{:#?}",
        x > pali_language_services::alphabet::PaliAlphabet::BH
    );

    let html = pali_language_services::inflections::generate_inflection_table(
        "ābādheti",
        get_pali1_metadata,
        get_itable,
        exec_isql,
    )
    .unwrap();
    println!("{:#?}", html);

    std::fs::write("d:/delme/inflections.txt", &html).unwrap();
}
