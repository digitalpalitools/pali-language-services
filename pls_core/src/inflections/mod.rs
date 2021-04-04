mod generators;

use crate::alphabet::string_compare;
use regex::{Error, Regex};
use serde::Serialize;
use std::collections::HashMap;
use tera::{Context, Tera};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![("output", include_str!("templates/output.html"))])
            .expect("Unexpected failure adding template");
        tera.autoescape_on(vec!["html"]);
        tera
    };
    static ref INDECLINABLE_CRACKER: Result<Regex, Error> = Regex::new(r" \d+$");
}

#[derive(Debug)]
pub enum InflectionClass {
    Indeclinable,
    Conjugation,
    Declension,
    DeclensionPron1st,
    DeclensionPron2nd,
    DeclensionPronDual,
}

pub struct Pali1Metadata {
    pub pali1: String,
    pub stem: String,
    pub pattern: String,
    pub pos: String,
    pub meaning: String,
    pub inflection_class: InflectionClass,
    pub like: String,
}

pub struct SqlQuery {
    exec_sql_query: fn(&str) -> Result<String, String>,
}

impl SqlQuery {
    fn new(exec_sql_query: fn(&str) -> Result<String, String>) -> SqlQuery {
        SqlQuery { exec_sql_query }
    }

    fn exec(&self, query: &str) -> Result<Vec<Vec<Vec<String>>>, String> {
        let result_str = (self.exec_sql_query)(&query)?;
        let result: Vec<Vec<Vec<String>>> =
            serde_json::from_str(&result_str).map_err(|e| e.to_string())?;
        Ok(result)
    }
}

pub fn generate_inflection_table(
    pali1: &str,
    host_url: &str,
    host_version: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql_query: fn(&str) -> Result<String, String>,
    locale: &str,
) -> Result<String, String> {
    let q = SqlQuery::new(exec_sql_query);
    let pm = get_pali1_metadata(pali1, transliterate, &q)?;
    let body = generators::create_html_body(&pm, transliterate, &q, locale)?;

    generate_output(&pm, pali1, host_url, host_version, &body, transliterate)
}

fn inflection_class_from_str(ic: &str) -> InflectionClass {
    match ic {
        "verb" => InflectionClass::Conjugation,
        "pron1st" => InflectionClass::DeclensionPron1st,
        "pron2nd" => InflectionClass::DeclensionPron2nd,
        "prondual" => InflectionClass::DeclensionPronDual,
        _ => InflectionClass::Declension,
    }
}

fn get_stem_for_indeclinable(pali1: &str) -> Result<String, String> {
    let regex = INDECLINABLE_CRACKER.as_ref().map_err(|e| e.to_string())?;
    Ok(regex.replace(pali1, "").to_string())
}

fn get_pali1_metadata(
    pali1: &str,
    transliterate: fn(&str) -> Result<String, String>,
    q: &SqlQuery,
) -> Result<Pali1Metadata, String> {
    let sql = format!(
        r#"select stem, pattern, pos, definition from '_stems' where pāli1 = "{}""#,
        pali1,
    );
    let results = q.exec(&sql)?;
    let stem = &results[0][0][0];
    let pattern = &results[0][0][1];
    let mut pm = Pali1Metadata {
        pali1: pali1.to_string(),
        stem: if !stem.eq("*") {
            stem.to_owned()
        } else {
            "".to_string()
        },
        pattern: pattern.to_owned(),
        pos: results[0][0][2].to_owned(),
        meaning: results[0][0][3].to_owned(),
        inflection_class: InflectionClass::Declension,
        like: "".to_string(),
    };

    if !pattern.trim().is_empty() {
        let sql = format!(
            r#"select inflection_class, like from '_index' where name = "{}""#,
            pattern
        );
        let results = q.exec(&sql)?;
        let inflection_class = &results[0][0][0];
        let like = &results[0][0][1];

        pm.inflection_class = inflection_class_from_str(inflection_class);
        pm.like = if !like.is_empty() {
            format!("like {}", transliterate(like)?)
        } else {
            "irregular".to_string()
        };
    } else if stem.eq("-") {
        pm.inflection_class = InflectionClass::Indeclinable;
        pm.pali1 = get_stem_for_indeclinable(&pm.pali1)?;
        pm.like = "indeclinable".to_string();
    }

    Ok(pm)
}

#[derive(Serialize)]
struct ViewModel<'a> {
    pub pali1: &'a str,
    pub pattern: &'a str,
    pub like: &'a str,
    pub pos: &'a str,
    pub meaning: &'a str,
    pub body: &'a str,
    pub feedback_form_url: &'a str,
    pub host_url: &'a str,
    pub host_version: &'a str,
}

fn generate_output(
    pm: &Pali1Metadata,
    pali1: &str,
    host_url: &str,
    host_version: &str,
    body: &str,
    transliterate: fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    let feedback_form_url = match pm.inflection_class {
        InflectionClass::Conjugation => {
            "https://docs.google.com/forms/d/e/1FAIpQLSeJpx7TsISkYEXzxvbBtOH25T-ZO1Z5NFdujO5SD9qcAH_i1A/viewform"
        }
        _ => { // All declensions.
            "https://docs.google.com/forms/d/e/1FAIpQLSeoxZiqvIWadaLeuXF4f44NCqEn49-B8KNbSvNer5jxgRYdtQ/viewform"
        }
    };

    let vm = ViewModel {
        pali1: &transliterate(pali1)?,
        pattern: &pm.pattern,
        like: &pm.like,
        pos: &pm.pos,
        meaning: &pm.meaning,
        body: &body,
        feedback_form_url: &feedback_form_url,
        host_url: &host_url,
        host_version: &host_version,
    };

    let context = Context::from_serialize(&vm).map_err(|e| e.to_string())?;
    TEMPLATES
        .render("output", &context)
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone)]
pub struct InflectedWordMetadata {
    pub inflected_word: String,
    pub stem_word: String,
    pub grammar: String,
    pub comment: String,
}

impl InflectedWordMetadata {
    pub fn simple_representation(self) -> (String, String, String, String) {
        (
            self.inflected_word,
            self.stem_word,
            self.grammar,
            self.comment,
        )
    }
}

fn get_inflections_for_pattern(
    pattern: &str,
    q: &SqlQuery,
) -> Result<Vec<Vec<Vec<String>>>, String> {
    q.exec(&format!("Select * from {}", pattern))
}

fn get_words_for_indeclinable_stem(pali1: &str) -> Result<Vec<InflectedWordMetadata>, String> {
    Ok(vec![InflectedWordMetadata {
        inflected_word: get_stem_for_indeclinable(pali1)?,
        stem_word: pali1.to_string(),
        grammar: " ".to_string(),
        comment: "ind".to_string(),
    }])
}

fn get_words_for_irregular_stem(
    pali1: &str,
    pattern: &str,
    q: &SqlQuery,
) -> Result<Vec<InflectedWordMetadata>, String> {
    let inflections: Vec<Vec<String>> = get_inflections_for_pattern(pattern, &q)?
        .pop()
        .ok_or_else(|| format!("No pattern found for {}", pattern))?;
    let mut inflected_words_irregular_stem: Vec<InflectedWordMetadata> = Vec::new();
    for mut inflection_row in inflections {
        for inflection in inflection_row
            .pop()
            .ok_or_else(|| format!("No pattern found for {}", pattern))?
            .split(',')
        {
            inflected_words_irregular_stem.push(InflectedWordMetadata {
                inflected_word: inflection.to_string(),
                stem_word: pali1.to_string(),
                grammar: inflection_row.join(" ").to_string(),
                comment: "*".to_string(),
            })
        }
    }
    Ok(inflected_words_irregular_stem)
}

fn get_words_for_regular_stem(
    pali1: &str,
    stem: &str,
    pattern: &str,
    q: &SqlQuery,
) -> Result<Vec<InflectedWordMetadata>, String> {
    let mut inflected_words_regular_stem: Vec<InflectedWordMetadata> = Vec::new();
    let inflections: Vec<Vec<String>> = get_inflections_for_pattern(pattern, &q)?
        .pop()
        .ok_or_else(|| format!("No pattern found for {}", pattern))?;
    for mut inflection_row in inflections {
        for inflection in inflection_row
            .pop()
            .ok_or_else(|| format!("No pattern found for {}", pattern))?
            .split(',')
        {
            inflected_words_regular_stem.push(InflectedWordMetadata {
                inflected_word: [stem, inflection].join("").to_string(),
                stem_word: pali1.to_string(),
                grammar: inflection_row.join(" ").to_string(),
                comment: " ".to_string(),
            })
        }
    }
    Ok(inflected_words_regular_stem)
}

pub fn generate_all_inflected_words(
    pali1: &str,
    stem: &str,
    pattern: &str,
    exec_sql_query: fn(&str) -> Result<String, String>,
) -> Result<Vec<InflectedWordMetadata>, String> {
    let q = SqlQuery::new(exec_sql_query);
    let inflected_words: Vec<InflectedWordMetadata> = match stem {
        "-" => get_words_for_indeclinable_stem(pali1)?,
        "*" => get_words_for_irregular_stem(pali1, pattern, &q)?,
        _ => get_words_for_regular_stem(pali1, stem, pattern, &q)?,
    };
    Ok(inflected_words)
}

fn join_and_transliterate_if_not_empty(
    stem: &str,
    suffix: &str,
    transliterate: fn(&str) -> Result<String, String>,
) -> String {
    if suffix.is_empty() {
        "".to_string()
    } else {
        transliterate(&format!("{}{}", stem, suffix)).unwrap_or_else(|e| e)
    }
}

fn get_inflections(
    stem: &str,
    sql: &str,
    transliterate: fn(&str) -> Result<String, String>,
    q: &SqlQuery,
) -> Vec<String> {
    let res = match q.exec(&sql) {
        Ok(x) => {
            if x.len() == 1 && x[0].len() == 1 && x[0][0].len() == 1 {
                x[0][0][0].to_string()
            } else {
                "".to_string()
            }
        }
        Err(e) => e,
    };

    let mut inflections: Vec<String> = res
        .split(',')
        .map(|s| join_and_transliterate_if_not_empty(stem, s, transliterate))
        .collect();
    inflections.sort_by(|a, b| Ord::cmp(&string_compare(a, b), &0));
    inflections
}

fn query_has_no_results(query: &str, q: &SqlQuery) -> Result<bool, String> {
    let count = &q.exec(&query)?[0][0][0];
    Ok(count.eq("0"))
}

pub fn get_abbreviations_for_locale(
    locale: &str,
    q: &SqlQuery,
) -> Result<HashMap<String, String>, String> {
    let sql: String;
    if locale == "xx" {
        sql = "select name, description, '^' || name || '$' from _abbreviations".to_string();
    } else if locale == "en" {
        sql = "select name, description, name from _abbreviations".to_string();
    } else {
        sql = format!(
            r#"select name, description, {} from _abbreviations"#,
            locale
        );
    }
    let res = q.exec(&sql)?;
    let mut abbrev_map = HashMap::new();
    for i in res[0].iter() {
        abbrev_map.insert(i[0].clone(), i[2].clone());
        abbrev_map.insert(i[1].clone(), i[2].clone());
    }
    let list_of_required_keys = [
        "nom", "acc", "instr", "dat", "abl", "gen", "loc", "voc", "in comps", "masc", "fem", "nt",
        "x", "sg", "pl", "dual", "act", "reflx", "pr", "fut", "aor", "opt", "imp", "cond",
        "imperf", "perf", "1st", "2nd", "3rd", "irreg", "gram", "ind", "abs", "adj", "adv", "base",
        "card", "case", "comp", "cs", "ger", "idiom", "inf", "like", "ordin", "person", "pp",
        "prefix", "pron", "prp", "ptp", "root", "sandhi", "suffix", "ve",
    ];
    for i in list_of_required_keys.iter() {
        if !abbrev_map.contains_key(&i.to_string()) {
            abbrev_map.insert(i.to_string(), format!("{} ABBREVIATION NOT FOUND", i));
        }
    }
    Ok(abbrev_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{Connection, Row, NO_PARAMS};
    use test_case::test_case;

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

    fn psuedo_transliterate(s: &str) -> String {
        format!("^{}$", s)
    }

    #[test_case("ābādheti","xx"; "conjugation - 1 - xx")]
    #[test_case("vassūpanāyikā","xx"; "declension - 1 - xx ")]
    #[test_case("kamma 1","xx"; "declension - 2 - irreg - xx")]
    #[test_case("kāmaṃ 3","xx"; "declension - 3 - ind - xx")]
    #[test_case("ubha","xx"; "declension - 4 - pron_dual - xx")]
    #[test_case("maṃ","xx"; "declension - 4 - pron_1st - xx")]
    #[test_case("taṃ 3","xx"; "declension - 4 - pron_2nd - xx")]
    #[test_case("pañca","xx"; "declension - 5 - only x gender - xx")]
    #[test_case("ābādheti","en"; "conjugation - 1 - en")]
    fn inflection_tests(pali1: &str, locale: &str) {
        let html = generate_inflection_table(
            pali1,
            "test case",
            "v0.1",
            |s| Ok(psuedo_transliterate(s)),
            exec_sql,
            locale,
        )
        .unwrap_or_else(|e| e);
        insta::assert_snapshot!(html);
    }

    #[test_case("a 1", "-", ""; "indeclinable")]
    #[test_case("ababa 1", "abab", "a_nt"; "regular")]
    #[test_case("ahesuṃ", "*", "ahosi_aor"; "irregular")]
    fn inflected_word_tests(pali1: &str, stem: &str, pattern: &str) {
        let output: Vec<(String, String, String, String)> =
            generate_all_inflected_words(pali1, stem, pattern, exec_sql)
                .unwrap_or_else(|_e| Vec::new())
                .iter_mut()
                .map(|x| x.clone().simple_representation())
                .collect();

        insta::assert_yaml_snapshot!(output);
    }
}
