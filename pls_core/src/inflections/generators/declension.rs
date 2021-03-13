use crate::inflections;
use crate::inflections::{generators, SqlQuery};
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
    inflections_list: Vec<Vec<String>>,
}

#[derive(Serialize)]
struct TemplateViewModel<'a> {
    pattern: &'a str,
    stem: &'a str,
    view_models: Vec<CaseViewModel>,
    in_comps_inflections: Vec<String>,
}

pub fn create_html_body(
    pattern: &str,
    stem: &str,
    transliterate: fn(&str) -> Result<String, String>,
    q: &SqlQuery,
) -> Result<String, String> {
    let table_name = &generators::get_table_name_from_pattern(pattern);
    let view_models = create_case_view_models(&table_name, transliterate, &q, stem)?;
    let in_comps_inflections =
        create_template_view_model_for_in_comps(table_name, transliterate, &q, stem);

    let template_view_model = TemplateViewModel {
        pattern,
        stem,
        view_models,
        in_comps_inflections,
    };

    let context = Context::from_serialize(&template_view_model).map_err(|e| e.to_string())?;
    TEMPLATES
        .render("declension", &context)
        .map_err(|e| e.to_string())
}

fn create_case_view_models(
    table_name: &str,
    transliterate: fn(&str) -> Result<String, String>,
    q: &SqlQuery,
    stem: &str,
) -> Result<Vec<CaseViewModel>, String> {
    let sql = r#"
        select * from _case_values where name <> "";
        select * from _gender_values where name <> "" and name <> "x";
        select * from _number_values where name <> "" and name <> "dual";
    "#;
    let values = q.exec(sql)?;
    let mut view_models: Vec<CaseViewModel> = Vec::new();
    for case in values[0].iter().flatten() {
        let mut inflections_list: Vec<Vec<String>> = Vec::new();
        for gender in values[1].iter().flatten() {
            for number in values[2].iter().flatten() {
                let sql = format!(
                    r#"SELECT inflections FROM '{}' WHERE "case" = '{}' AND gender = '{}' AND "number" = '{}'"#,
                    table_name, case, gender, number
                );
                let inflections = inflections::get_inflections(&stem, &sql, transliterate, &q);
                inflections_list.push(inflections);
            }
        }

        let view_model = CaseViewModel {
            name: case.to_owned(),
            inflections_list,
        };
        view_models.push(view_model);
    }

    Ok(view_models)
}

fn create_template_view_model_for_in_comps(
    table_name: &str,
    transliterate: fn(&str) -> Result<String, String>,
    q: &SqlQuery,
    stem: &str,
) -> Vec<String> {
    let sql = format!(
        r#"SELECT inflections FROM '{}' WHERE "case" = '' AND gender = '' AND "number" = ''"#,
        table_name
    );

    inflections::get_inflections(&stem, &sql, transliterate, &q)
}
