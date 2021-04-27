use crate::inflections;
use crate::inflections::{localise_abbrev, PlsInflectionsHost};
use tera::{Context, Tera};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.register_filter("localise_abbrev", localise_abbrev);
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
    word_type: &str,
    stem: &str,
    host: &dyn PlsInflectionsHost,
) -> Result<(String, bool), String> {
    let mut context = Context::new();
    let abbrev_map = inflections::get_abbreviations_for_locale(host)?;

    context.insert("stem", &host.transliterate(stem)?);
    context.insert("word_type", word_type);
    context.insert("abbrev_map", &abbrev_map);
    let body = TEMPLATES
        .render("indeclinable", &context)
        .map_err(|e| e.to_string())?;

    Ok((body, false))
}
