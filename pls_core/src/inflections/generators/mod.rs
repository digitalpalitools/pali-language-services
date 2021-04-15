use crate::inflections::{InflectionClass, Pali1Metadata, PlsInflectionsHost};

mod conjugation;
mod declension;
mod declension_pron_dual;
mod declension_pron_x;
mod indeclinable;

pub fn create_html_body(
    pm: &Pali1Metadata,
    host: &dyn PlsInflectionsHost,
) -> Result<String, String> {
    match pm.inflection_class {
        InflectionClass::Indeclinable => indeclinable::create_html_body(&pm.pali1, host),
        InflectionClass::Conjugation => conjugation::create_html_body(&pm.pattern, &pm.stem, host),
        InflectionClass::Declension => declension::create_html_body(&pm.pattern, &pm.stem, host),
        InflectionClass::DeclensionPron1st => {
            declension_pron_x::create_html_body("1st", &pm.pattern, &pm.stem, host)
        }
        InflectionClass::DeclensionPron2nd => {
            declension_pron_x::create_html_body("2nd", &pm.pattern, &pm.stem, host)
        }
        InflectionClass::DeclensionPronDual => {
            declension_pron_dual::create_html_body(&pm.pattern, &pm.stem, host)
        }
    }
}

fn get_table_name_from_pattern(pattern: &str) -> String {
    pattern.replace(" ", "_")
}
