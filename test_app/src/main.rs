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
    use std::fs;
    use test_case::test_case;

    #[test_case("ābādheti", "conjugation - 1"; "conjugation - 1")]
    #[test_case("vassūpanāyikā", "declension - 1"; "declension - 1")]
    #[test_case("kamma 1", "declension - 2 - irreg"; "declension - 2 - irreg")]
    // #[test_case("kāmaṃ 3", "declension - 3 - ind"; "declension - 3 - ind")]
    // #[test_case("ubha", "declension - 4 - pron_dual"; "declension - 4 - pron_dual")]
    // #[test_case("ahaṃ", "declension - 4 - pron_1st"; "declension - 4 - pron_1st")]
    // #[test_case("taṃ 3", "declension - 4 - pron_2nd"; "declension - 4 - pron_2nd")]
    // TODO: Need to be abstracted, granularized and moved to pls_core.
    fn inflection_tests(pali1: &str, approved_filename: &str) -> Result<(), String> {
        let html = pls_core::inflections::generate_inflection_table(
            pali1,
            |s| Ok(s.to_string()),
            exec_sql,
        )?;

        let approved_html =
            fs::read_to_string(format!("src/test_data/{}.approved.html", approved_filename))
                .map_err(|e| e.to_string())?;

        fs::write(
            format!("src/test_data/{}.actual.html", approved_filename),
            &html,
        )
        .map_err(|e| e.to_string())?;

        assert_eq!(html, approved_html);

        Ok(())
    }

    #[test]
    fn inflected_word_indeclinable_test() {
        let output: Vec<(String, String, String, String)> =
            pls_core::inflections::generate_all_inflected_words("a 1", "-", "", exec_sql)
                .unwrap()
                .iter_mut()
                .map(|x| x.clone().simple_representation())
                .collect();
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
        let output: Vec<(String, String, String, String)> =
            pls_core::inflections::generate_all_inflected_words(
                "ababa 1", "abab", "a_nt", exec_sql,
            )
            .unwrap()
            .iter()
            .map(|x| x.clone().simple_representation())
            .collect();
        let expected: Vec<(String, String, String, String)> = [
            ("ababesu", "ababa 1", "nt loc pl", " "),
            ("ababāya", "ababa 1", "nt dat sg", " "),
            ("ababassa", "ababa 1", "nt dat sg", " "),
            ("ababe", "ababa 1", "nt acc pl", " "),
            ("ababāni", "ababa 1", "nt acc pl", " "),
            ("ababena", "ababa 1", "nt instr sg", " "),
            ("ababānaṃ", "ababa 1", "nt dat pl", " "),
            ("ababā", "ababa 1", "nt abl sg", " "),
            ("ababamhā", "ababa 1", "nt abl sg", " "),
            ("ababasmā", "ababa 1", "nt abl sg", " "),
            ("ababato", "ababa 1", "nt abl sg", " "),
            ("ababa", "ababa 1", "  ", " "),
            ("ababā", "ababa 1", "nt nom pl", " "),
            ("ababāni", "ababa 1", "nt nom pl", " "),
            ("ababānaṃ", "ababa 1", "nt gen pl", " "),
            ("ababaṃ", "ababa 1", "nt acc sg", " "),
            ("ababaṃ", "ababa 1", "nt nom sg", " "),
            ("ababe", "ababa 1", "nt loc sg", " "),
            ("ababamhi", "ababa 1", "nt loc sg", " "),
            ("ababasmiṃ", "ababa 1", "nt loc sg", " "),
            ("ababehi", "ababa 1", "nt instr pl", " "),
            ("ababebhi", "ababa 1", "nt instr pl", " "),
            ("ababassa", "ababa 1", "nt gen sg", " "),
            ("ababehi", "ababa 1", "nt abl pl", " "),
            ("ababebhi", "ababa 1", "nt abl pl", " "),
            ("ababato", "ababa 1", "nt abl pl", " "),
            ("ababa", "ababa 1", "nt voc sg", " "),
            ("ababā", "ababa 1", "nt voc sg", " "),
            ("ababaṃ", "ababa 1", "nt voc sg", " "),
            ("ababā", "ababa 1", "nt voc pl", " "),
            ("ababāni", "ababa 1", "nt voc pl", " "),
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
        let output: Vec<(String, String, String, String)> =
            pls_core::inflections::generate_all_inflected_words(
                "ahesuṃ",
                "*",
                "ahosi_aor",
                exec_sql,
            )
            .unwrap()
            .iter()
            .map(|x| x.clone().simple_representation())
            .collect();
        let expected: Vec<(String, String, String, String)> = [
            ("ahuvattha", "ahesuṃ", "act aor 2nd pl", "*"),
            ("ahosittha", "ahesuṃ", "act aor 2nd pl", "*"),
            ("ahosi", "ahesuṃ", "act aor 3rd sg", "*"),
            ("ahu", "ahesuṃ", "act aor 3rd sg", "*"),
            ("ahuvā", "ahesuṃ", "reflx aor 3rd sg", "*"),
            ("ahuvivhaṃ", "ahesuṃ", "reflx aor 2nd pl", "*"),
            ("ahuvimhe", "ahesuṃ", "reflx aor 1st pl", "*"),
            ("ahosiṃ", "ahesuṃ", "act aor 1st sg", "*"),
            ("ahuṃ", "ahesuṃ", "act aor 1st sg", "*"),
            ("ahuvāsiṃ", "ahesuṃ", "act aor 1st sg", "*"),
            ("ahumha", "ahesuṃ", "act aor 1st pl", "*"),
            ("ahumhā", "ahesuṃ", "act aor 1st pl", "*"),
            ("ahosimha", "ahesuṃ", "act aor 1st pl", "*"),
            ("ahuvase", "ahesuṃ", "reflx aor 2nd sg", "*"),
            ("ahesuṃ", "ahesuṃ", "act aor 3rd pl", "*"),
            ("ahuṃ", "ahesuṃ", "act aor 3rd pl", "*"),
            ("ahuvo", "ahesuṃ", "act aor 2nd sg", "*"),
            ("ahosi", "ahesuṃ", "act aor 2nd sg", "*"),
            ("ahuvaṃ", "ahesuṃ", "reflx aor 1st sg", "*"),
            ("ahuṃ", "ahesuṃ", "reflx aor 1st sg", "*"),
            ("ahuvu", "ahesuṃ", "reflx aor 3rd pl", "*"),
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
