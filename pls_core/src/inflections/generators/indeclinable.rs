use crate::inflections::{localise_abbrev, PlsInflectionsHost, pmd::get_pali1_metadata};
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
    word: &str,
    is_inflected_form: bool,
    host: &dyn PlsInflectionsHost,
) -> Result<(String, bool), String> {
    let mut context = Context::new();
    let pm = get_pali1_metadata(&word, host)?;

    context.insert("word", &host.transliterate(word)?);
    context.insert("is_inflected_form", &is_inflected_form);
    context.insert("meaning", &pm.meaning);
    context.insert("pos", &pm.pos);
    let body = TEMPLATES
        .render("indeclinable", &context)
        .map_err(|e| e.to_string())?;

    Ok((body, false))
}
