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
        "bala 1",
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

    #[test]
    fn inflected_word_indeclinable_test() {
        let output:Vec<(String, String, String, String)> =
        pls_core::inflections::generate_all_inflected_words("a 1", "-", "", exec_sql).unwrap().iter_mut().map(|x| x.clone().simple_representation()).collect();
        let expected: Vec<(String, String, String, String)> = [("a ", "a 1", " ", "ind")]
            .iter()
            .map(|x| {
                (
                    x.0.to_string(),
                    x.1.to_string(),
                    x.2.to_string(),
                    x.3.to_string(),
                )
            })
            .collect();
        assert_eq!(output, expected);
    }

    #[test]
    fn inflected_word_regular_test() {
        let output:Vec<(String, String, String, String)> = pls_core::inflections::generate_all_inflected_words(
            "ababa 1", "abab", "a_nt", exec_sql,
        )
        .unwrap().iter().map(|x| x.clone().simple_representation()).collect();
        let expected: Vec<(String, String, String, String)> = [
            ("ababā", "ababa 1", "nt nom pl", " "),
            ("ababāni", "ababa 1", "nt nom pl", " "),
            ("ababānaṃ", "ababa 1", "nt gen pl", " "),
            ("ababesu", "ababa 1", "nt loc pl", " "),
            ("ababāya", "ababa 1", "nt dat sg", " "),
            ("ababassa", "ababa 1", "nt dat sg", " "),
            ("ababa", "ababa 1", "  ", " "),
            ("ababaṃ", "ababa 1", "nt nom sg", " "),
            ("ababā", "ababa 1", "nt voc pl", " "),
            ("ababāni", "ababa 1", "nt voc pl", " "),
            ("ababehi", "ababa 1", "nt instr pl", " "),
            ("ababebhi", "ababa 1", "nt instr pl", " "),
            ("ababa", "ababa 1", "nt voc sg", " "),
            ("ababā", "ababa 1", "nt voc sg", " "),
            ("ababaṃ", "ababa 1", "nt voc sg", " "),
            ("ababassa", "ababa 1", "nt gen sg", " "),
            ("ababe", "ababa 1", "nt acc pl", " "),
            ("ababāni", "ababa 1", "nt acc pl", " "),
            ("ababe", "ababa 1", "nt loc sg", " "),
            ("ababamhi", "ababa 1", "nt loc sg", " "),
            ("ababasmiṃ", "ababa 1", "nt loc sg", " "),
            ("ababena", "ababa 1", "nt instr sg", " "),
            ("ababā", "ababa 1", "nt abl sg", " "),
            ("ababamhā", "ababa 1", "nt abl sg", " "),
            ("ababasmā", "ababa 1", "nt abl sg", " "),
            ("ababato", "ababa 1", "nt abl sg", " "),
            ("ababaṃ", "ababa 1", "nt acc sg", " "),
            ("ababānaṃ", "ababa 1", "nt dat pl", " "),
            ("ababehi", "ababa 1", "nt abl pl", " "),
            ("ababebhi", "ababa 1", "nt abl pl", " "),
            ("ababato", "ababa 1", "nt abl pl", " "),
        ]
        .iter()
        .map(|x| {
            (
                x.0.to_string(),
                x.1.to_string(),
                x.2.to_string(),
                x.3.to_string(),
            )
        })
        .collect();
        assert_eq!(output, expected);
    }

    #[test]
    fn inflected_word_irregular_test() {
        let output:Vec<(String, String, String, String)> = pls_core::inflections::generate_all_inflected_words(
            "ahesuṃ",
            "*",
            "ahosi_aor",
            exec_sql,
        )
        .unwrap().iter().map(|x| x.clone().simple_representation()).collect();
        let expected: Vec<(String, String, String, String)> = [
            ("ahuvattha", "ahesuṃ", "act aor 2nd pl", "*"),
            ("ahosittha", "ahesuṃ", "act aor 2nd pl", "*"),
            ("ahosi", "ahesuṃ", "act aor 3rd sg", "*"),
            ("ahu", "ahesuṃ", "act aor 3rd sg", "*"),
            ("ahuvivhaṃ", "ahesuṃ", "reflx aor 2nd pl", "*"),
            ("ahesuṃ", "ahesuṃ", "act aor 3rd pl", "*"),
            ("ahuṃ", "ahesuṃ", "act aor 3rd pl", "*"),
            ("ahuvā", "ahesuṃ", "reflx aor 3rd sg", "*"),
            ("ahuvaṃ", "ahesuṃ", "reflx aor 1st sg", "*"),
            ("ahuṃ", "ahesuṃ", "reflx aor 1st sg", "*"),
            ("ahuvu", "ahesuṃ", "reflx aor 3rd pl", "*"),
            ("ahosiṃ", "ahesuṃ", "act aor 1st sg", "*"),
            ("ahuṃ", "ahesuṃ", "act aor 1st sg", "*"),
            ("ahuvāsiṃ", "ahesuṃ", "act aor 1st sg", "*"),
            ("ahuvimhe", "ahesuṃ", "reflx aor 1st pl", "*"),
            ("ahuvo", "ahesuṃ", "act aor 2nd sg", "*"),
            ("ahosi", "ahesuṃ", "act aor 2nd sg", "*"),
            ("ahuvase", "ahesuṃ", "reflx aor 2nd sg", "*"),
            ("ahumha", "ahesuṃ", "act aor 1st pl", "*"),
            ("ahumhā", "ahesuṃ", "act aor 1st pl", "*"),
            ("ahosimha", "ahesuṃ", "act aor 1st pl", "*"),
        ]
        .iter()
        .map(|x| {
            (
                x.0.to_string(),
                x.1.to_string(),
                x.2.to_string(),
                x.3.to_string(),
            )
        })
        .collect();
        assert_eq!(output, expected);
    }
}
