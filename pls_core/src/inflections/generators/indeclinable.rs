use tera::{Context, Tera};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![(
            "indeclinable",
            include_str!("templates/indeclinable.html"),
        )])
        .unwrap();
        tera.autoescape_on(vec!["html", ".sql"]);
        tera
    };
}

pub fn create_html_body(
    pali1: &str,
    transliterate: fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    let mut context = Context::new();
    context.insert("inflection", &transliterate(pali1)?);

    TEMPLATES
        .render("indeclinable", &context)
        .map_err(|e| e.to_string())
}
