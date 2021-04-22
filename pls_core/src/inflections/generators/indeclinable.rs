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

pub fn create_html_body(pali1: &str, host: &dyn PlsInflectionsHost) -> Result<String, String> {
    let mut context = Context::new();
    let abbrev_map = inflections::get_abbreviations_for_locale(host)?;

    context.insert("inflection", &host.transliterate(pali1)?);
    context.insert("abbrev_map", &abbrev_map);
    TEMPLATES
        .render("indeclinable", &context)
        .map_err(|e| e.to_string())
}
