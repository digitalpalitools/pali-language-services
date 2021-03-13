use crate::inflections::{InflectionClass, Pali1Metadata};

mod conjugation;
mod declension;
mod declension_pron_dual;
mod declension_pron_x;
mod indeclinable;

pub fn create_html_body(
    pm: &Pali1Metadata,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<String, String> {
    match pm.inflection_class {
        InflectionClass::Indeclinable => indeclinable::create_html_body(&pm.pali1, transliterate),
        InflectionClass::Conjugation => {
            conjugation::create_html_body(&pm.pattern, &pm.stem, transliterate, exec_sql)
        }
        InflectionClass::Declension => {
            declension::create_html_body(&pm.pattern, &pm.stem, transliterate, exec_sql)
        }
        InflectionClass::DeclensionPron1st => declension_pron_x::create_html_body(
            "1st",
            &pm.pattern,
            &pm.stem,
            transliterate,
            exec_sql,
        ),
        InflectionClass::DeclensionPron2nd => declension_pron_x::create_html_body(
            "2nd",
            &pm.pattern,
            &pm.stem,
            transliterate,
            exec_sql,
        ),
        InflectionClass::DeclensionPronDual => {
            declension_pron_dual::create_html_body(&pm.pattern, &pm.stem, transliterate, exec_sql)
        }
    }
}

fn get_table_name_from_pattern(pattern: &str) -> String {
    pattern.replace(" ", "_")
}
