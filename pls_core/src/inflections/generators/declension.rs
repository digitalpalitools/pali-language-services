use crate::inflections;
use serde::Serialize;
use tera::{Context, Tera};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![(
            "declension",
            include_str!("templates/declension.html"),
        )])
        .unwrap();
        tera.autoescape_on(vec!["html", ".sql"]);
        tera
    };
}

#[derive(Serialize)]
struct CaseViewModel {
    name: String,
    stemmed_inflections_list: Vec<Vec<String>>,
}

#[derive(Serialize)]
struct TemplateViewModel<'a> {
    table_name: &'a str,
    stem: &'a str,
    case_view_models: Vec<CaseViewModel>,
    in_stemmed_comps_inflections: Vec<String>,
}

pub fn create_html_body(
    table_name: &str,
    stem: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<String, String> {
    let case_view_models = create_template_view_model(&table_name, transliterate, &exec_sql, stem)?;
    let in_stemmed_comps_inflections =
        create_template_view_model_for_in_comps(&table_name, transliterate, &exec_sql, stem)?;

    let vm = TemplateViewModel {
        table_name,
        stem,
        case_view_models,
        in_stemmed_comps_inflections,
    };

    let context = Context::from_serialize(&vm).map_err(|e| e.to_string())?;
    TEMPLATES
        .render("declension", &context)
        .map_err(|e| e.to_string())
}

fn create_template_view_model(
    table_name: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
    stem: &str,
) -> Result<Vec<CaseViewModel>, String> {
    let sql = r#"
        select * from _case_values where name <> "";
        select * from _gender_values where name <> "" and name <> "x";
        select * from _number_values where name <> "" and name <> "dual";
    "#;
    let values = exec_sql(sql)?;
    let mut view_models: Vec<CaseViewModel> = Vec::new();
    for case in values[0].iter().flatten() {
        let mut stemmed_inflections_list: Vec<Vec<String>> = Vec::new();
        for gender in values[1].iter().flatten() {
            for number in values[2].iter().flatten() {
                let sql = format!(
                    r#"SELECT inflections FROM '{}' WHERE "case" = '{}' AND gender = '{}' AND "number" = '{}'"#,
                    table_name, case, gender, number
                );
                let stemmed_inflections =
                    inflections::get_inflections_stemmed(&sql, &exec_sql, &stem, transliterate)?;
                stemmed_inflections_list.push(stemmed_inflections);
            }
        }

        let view_model = CaseViewModel {
            name: case.to_owned(),
            stemmed_inflections_list,
        };
        view_models.push(view_model);
    }

    Ok(view_models)
}

fn create_template_view_model_for_in_comps(
    table_name: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
    stem: &str,
) -> Result<Vec<String>, String> {
    let sql = format!(
        r#"SELECT inflections FROM '{}' WHERE "case" = '' AND gender = '' AND "number" = ''"#,
        table_name
    );

    inflections::get_inflections_stemmed(&sql, &exec_sql, &stem, transliterate)
}
