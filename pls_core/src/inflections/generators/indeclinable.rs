use crate::inflections::pmd::{Pali1Metadata, WordType};
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
    pm: &Pali1Metadata,
    host: &dyn PlsInflectionsHost,
    with_details: bool,
) -> Result<(String, bool), String> {
    let mut context = Context::new();
    match &pm.word_type {
        WordType::InflectedForm { stems: stem } => {
            context.insert("word", &host.transliterate(stem)?);
            context.insert("is_inflected_form", &true);
        }
        WordType::Indeclinable { stem: _stem } => {
            context.insert("word", &host.transliterate(&pm.pali1)?);
            context.insert("is_inflected_form", &false);
        }
        _ => {
            return Err(format!(
                "WordType should be either InflectedForm or Indeclinable."
            ))
        }
    }
    context.insert("meaning", &pm.meaning);
    context.insert("pos", &pm.pos);
    context.insert("with_details", &with_details);
    let body = TEMPLATES
        .render("indeclinable", &context)
        .map_err(|e| e.to_string())?;

    Ok((body, false))
}
