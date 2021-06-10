use crate::inflections::pmd::{InflectionClass, Pali1Metadata, WordType};
use crate::inflections::PlsInflectionsHost;

mod conjugation;
mod declension;
mod declension_pron_dual;
mod declension_pron_x;
mod indeclinable;

pub fn create_html_body(
    pm: &Pali1Metadata,
    host: &dyn PlsInflectionsHost,
    with_details: bool,
) -> Result<(String, bool), String> {
    match &pm.word_type {
        WordType::InflectedForm { stems: stem } => {
            indeclinable::create_html_body(&stem, true, host, with_details)
        }
        WordType::Indeclinable { stem: _stem } => {
            indeclinable::create_html_body(&pm.pali1, false, host, with_details)
        }
        WordType::Irregular {
            pattern,
            inflection_class,
        } => create_html_body_for_inflection_class("", pattern, inflection_class, host),
        WordType::Declinable {
            stem,
            pattern,
            inflection_class,
        } => create_html_body_for_inflection_class(stem, pattern, inflection_class, host),
    }
}

fn create_html_body_for_inflection_class(
    stem: &str,
    pattern: &str,
    inflection_class: &InflectionClass,
    host: &dyn PlsInflectionsHost,
) -> Result<(String, bool), String> {
    let body = match inflection_class {
        InflectionClass::Conjugation => conjugation::create_html_body(pattern, stem, host),
        InflectionClass::Declension => declension::create_html_body(pattern, stem, host),
        InflectionClass::DeclensionPron1st => {
            declension_pron_x::create_html_body("1st", pattern, stem, host)
        }
        InflectionClass::DeclensionPron2nd => {
            declension_pron_x::create_html_body("2nd", pattern, stem, host)
        }
        InflectionClass::DeclensionPronDual => {
            declension_pron_dual::create_html_body(pattern, stem, host)
        }
    };

    Ok((body?, true))
}
