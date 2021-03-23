use crate::inflections;
use crate::inflections::SqlQuery;
use tera::{Context, Tera};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![(
            "indeclinable",
            include_str!("templates/indeclinable.html"),
        )])
        .expect("Unexpected failure adding template");
        tera.autoescape_on(vec!["html"]);
        tera
    };
}

pub fn create_html_body(
    pali1: &str,
    transliterate: fn(&str) -> Result<String, String>,
    q: &SqlQuery,
    locale: &str,
) -> Result<String, String> {
    let mut context = Context::new();
    let abbrev_map = inflections::get_abbreviations_for_locale(locale, q)?;

    context.insert("inflection", &transliterate(pali1)?);
    context.insert("abbrev_map", &abbrev_map);
    TEMPLATES
        .render("indeclinable", &context)
        .map_err(|e| e.to_string())
}
