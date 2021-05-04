use crate::inflections::host::PlsInflectionsHost;
use regex::{Error, Regex};
use serde::Serialize;
use std::str::FromStr;

lazy_static! {
    static ref INDECLINABLE_CRACKER: Result<Regex, Error> = Regex::new(r" \d+$");
}

#[derive(Debug, Serialize)]
pub enum WordType {
    InflectedForm {
        stems: String,
    },
    Indeclinable {
        stem: String,
    },
    Irregular {
        pattern: String,
        inflection_class: InflectionClass,
    },
    Declinable {
        stem: String,
        pattern: String,
        inflection_class: InflectionClass,
    },
}

#[derive(Debug, Serialize)]
pub enum InflectionClass {
    Conjugation,
    Declension,
    DeclensionPron1st,
    DeclensionPron2nd,
    DeclensionPronDual,
}

impl FromStr for InflectionClass {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "verb" => Ok(InflectionClass::Conjugation),
            "" => Ok(InflectionClass::Declension),
            "pron1st" => Ok(InflectionClass::DeclensionPron1st),
            "pron2nd" => Ok(InflectionClass::DeclensionPron2nd),
            "prondual" => Ok(InflectionClass::DeclensionPronDual),
            _ => Err(format!("Unknown inflection_class '{}'.", s)),
        }
    }
}

#[derive(Serialize)]
pub struct Pali1Metadata {
    pub pali1: String,
    pub word_type: WordType,
    pub pos: String,
    pub meaning: String,
    pub like: String,
    pub long_name: String,
}

pub fn get_stem_for_indeclinable(pali1: &str) -> Result<String, String> {
    let regex = INDECLINABLE_CRACKER.as_ref().map_err(|e| e.to_string())?;
    Ok(regex.replace(pali1, "").to_string())
}

pub fn get_pali1_metadata(
    pali1: &str,
    host: &dyn PlsInflectionsHost,
) -> Result<Pali1Metadata, String> {
    let sql = format!(
        r#"select stem, pattern, pos, definition from '_stems' where pāli1 = "{}""#,
        pali1,
    );
    let results = host.exec_sql_query(&sql)?;
    if results.len() != 1 || results[0].len() != 1 || results[0][0].len() != 4 {
        return Err(format!("Word '{}' not found in db.", pali1));
    }

    let stem = results[0][0][0].to_owned();
    let pattern = results[0][0][1].to_owned();
    let pos = results[0][0][2].to_owned();
    let meaning = results[0][0][3].to_owned();

    if stem.is_empty() {
        return Err("Stem cannot be empty".to_string());
    }

    let pm = match stem.as_str() {
        "!" => Pali1Metadata {
            pali1: pali1.to_string(),
            word_type: WordType::InflectedForm { stems: pattern },
            pos,
            meaning,
            like: "infl".to_string(),
            long_name: "inflected form".to_string(),
        },
        "-" => Pali1Metadata {
            pali1: pali1.to_string(),
            word_type: WordType::Indeclinable {
                stem: get_stem_for_indeclinable(pali1)?,
            },
            pos,
            meaning,
            like: "ind".to_string(),
            long_name: "indeclinable".to_string(),
        },
        "*" => {
            let (inflection_class, _) = get_index_info(&pattern, host)?;
            Pali1Metadata {
                pali1: pali1.to_string(),
                word_type: WordType::Irregular {
                    inflection_class,
                    pattern,
                },
                pos,
                meaning,
                like: "irreg".to_string(),
                long_name: "irregular".to_string(),
            }
        }
        _ => {
            let (inflection_class, like) = get_index_info(&pattern, host)?;
            Pali1Metadata {
                pali1: pali1.to_string(),
                word_type: WordType::Declinable {
                    stem,
                    inflection_class,
                    pattern,
                },
                pos,
                meaning,
                like: format!("like {}", host.transliterate(&like)?),
                long_name: "declinable".to_string(),
            }
        }
    };

    Ok(pm)
}

pub fn get_feedback_url_for_inflection_class(inflection_class: &InflectionClass) -> &str {
    match inflection_class {
        InflectionClass::Conjugation =>
            "https://docs.google.com/forms/d/e/1FAIpQLSeJpx7TsISkYEXzxvbBtOH25T-ZO1Z5NFdujO5SD9qcAH_i1A/viewform",
        _ =>
            "https://docs.google.com/forms/d/e/1FAIpQLSeoxZiqvIWadaLeuXF4f44NCqEn49-B8KNbSvNer5jxgRYdtQ/viewform",
    }
}

fn get_index_info(
    pattern: &str,
    host: &dyn PlsInflectionsHost,
) -> Result<(InflectionClass, String), String> {
    let sql = format!(
        r#"select inflection_class, like from '_index' where name = "{}""#,
        pattern
    );
    let results = host.exec_sql_query(&sql)?;
    let inflection_class = InflectionClass::from_str(&results[0][0][0])?;
    let like = results[0][0][1].to_owned();

    Ok((inflection_class, like))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inflections::test_host;
    use test_case::test_case;

    #[test_case("0xdeadbeef"; "not found")]
    #[test_case("ahesuṃ"; "inflected form")]
    #[test_case("a 1"; "indeclinable")]
    #[test_case("ababa 1"; "regular")]
    #[test_case("hoti 2"; "irregular")]
    fn get_pali1_metadata_tests(pali1: &str) {
        let host = test_host::Host {
            locale: "en",
            url: "test case",
            version: "v0.1",
            psuedo_transliterate: true,
        };

        let output = get_pali1_metadata(pali1, &host);

        insta::assert_yaml_snapshot!(output);
    }
}
