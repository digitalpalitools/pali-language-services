use crate::inflections;
use crate::inflections::{generators, localise_abbrev, PlsInflectionsHost};
use serde::Serialize;
use std::collections::HashMap;
use tera::{Context, Tera};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.register_filter("localise_abbrev", localise_abbrev);
        tera.add_raw_templates(vec![(
            "declension_pron_dual",
            include_str!("templates/declension_pron_dual.html"),
        )])
        .expect("Unexpected failure adding template");
        tera.autoescape_on(vec!["html"]);
        tera
    };
}

#[derive(Serialize)]
struct CaseViewModel {
    name: String,
    inflections: Vec<String>,
}

#[derive(Serialize)]
struct TemplateViewModel<'a> {
    pattern: &'a str,
    stem: &'a str,
    view_models: Vec<CaseViewModel>,
    in_comps_inflections: Vec<String>,
    abbrev_map: HashMap<String, String>,
}

pub fn create_html_body(
    pattern: &str,
    stem: &str,
    host: &dyn PlsInflectionsHost,
) -> Result<String, String> {
    let table_name = &generators::get_table_name_from_pattern(pattern);
    let view_models = create_case_view_models(table_name, &stem, host)?;
    let in_comps_inflections = create_template_view_model_for_in_comps(table_name, &stem, host);
    let abbrev_map = inflections::get_abbreviations_for_locale(host)?;

    let vm = TemplateViewModel {
        pattern,
        stem: &host.transliterate(stem)?,
        view_models,
        in_comps_inflections,
        abbrev_map,
    };

    let context = Context::from_serialize(&vm).map_err(|e| e.to_string())?;
    TEMPLATES
        .render("declension_pron_dual", &context)
        .map_err(|e| e.to_string())
}

fn create_case_view_models(
    table_name: &str,
    stem: &str,
    host: &dyn PlsInflectionsHost,
) -> Result<Vec<CaseViewModel>, String> {
    let sql = r#"select * from _case_values where name <> "" and name <> "voc""#;
    let values = host.exec_sql_query(sql)?;
    let mut view_models: Vec<CaseViewModel> = Vec::new();
    for case in values[0].iter().flatten() {
        let sql = format!(
            r#"SELECT inflections FROM '{}' WHERE "case" = '{}' AND special_pron_class = 'dual' AND "number" = 'sg'"#,
            table_name, case
        );
        let inflections = inflections::get_inflections(&stem, &sql, host);

        let view_model = CaseViewModel {
            name: case.to_owned(),
            inflections,
        };
        view_models.push(view_model);
    }

    Ok(view_models)
}

fn create_template_view_model_for_in_comps(
    table_name: &str,
    stem: &str,
    host: &dyn PlsInflectionsHost,
) -> Vec<String> {
    let sql = format!(
        r#"SELECT inflections FROM '{}' WHERE "case" = '' AND special_pron_class = '' AND "number" = ''"#,
        table_name
    );

    inflections::get_inflections(&stem, &sql, host)
}
