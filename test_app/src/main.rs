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
    use test_case::test_case;

    #[test_case("ābādheti"; "conjugation - 1")]
    #[test_case("vassūpanāyikā"; "declension - 1")]
    #[test_case("kamma 1"; "declension - 2 - irreg")]
    // #[test_case("kāmaṃ 3", "declension - 3 - ind"; "declension - 3 - ind")]
    #[test_case("ubha"; "declension - 4 - pron_dual")]
    #[test_case("maṃ"; "declension - 4 - pron_1st")]
    #[test_case("taṃ 3"; "declension - 4 - pron_2nd")]
    // TODO: Need to be abstracted, granularized and moved to pls_core.
    fn inflection_tests(pali1: &str) {
        let html = pls_core::inflections::generate_inflection_table(
            pali1,
            |s| Ok(s.to_string()),
            exec_sql,
        )
        .unwrap_or_else(|e| e);

        insta::assert_snapshot!(html);
    }

    #[test]
    fn inflected_word_indeclinable_test() {
        let output: Vec<(String, String, String, String)> =
            pls_core::inflections::generate_all_inflected_words("a 1", "-", "", exec_sql)
                .unwrap_or_else(|_e| Vec::new())
                .iter_mut()
                .map(|x| x.clone().simple_representation())
                .collect();

        insta::assert_yaml_snapshot!(output);
    }

    #[test]
    fn inflected_word_regular_test() {
        let output: Vec<(String, String, String, String)> =
            pls_core::inflections::generate_all_inflected_words(
                "ababa 1", "abab", "a_nt", exec_sql,
            )
            .unwrap_or_else(|_e| Vec::new())
            .iter()
            .map(|x| x.clone().simple_representation())
            .collect();

        insta::assert_yaml_snapshot!(output);
    }

    #[test]
    fn inflected_word_irregular_test() {
        let output: Vec<(String, String, String, String)> =
            pls_core::inflections::generate_all_inflected_words(
                "ahesuṃ",
                "*",
                "ahosi_aor",
                exec_sql,
            )
            .unwrap_or_else(|_e| Vec::new())
            .iter()
            .map(|x| x.clone().simple_representation())
            .collect();

        insta::assert_yaml_snapshot!(output);
    }
}
