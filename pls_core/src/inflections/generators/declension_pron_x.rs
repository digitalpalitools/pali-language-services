use crate::inflections;
use serde::Serialize;
use tera::{Context, Tera};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![(
            "declension_pron_x",
            include_str!("templates/declension_pron_x.html"),
        )])
        .unwrap();
        tera.autoescape_on(vec!["html", ".sql"]);
        tera
    };
}

#[derive(Serialize, Debug)]
struct CaseViewModel {
    name: String,
    stemmed_inflections_list: Vec<Vec<String>>,
}

#[derive(Serialize, Debug)]
struct TemplateViewModel<'a> {
    table_name: &'a str,
    pron_type: &'a str,
    stem: &'a str,
    view_models: Vec<CaseViewModel>,
    in_stemmed_comps_inflections: Vec<String>,
}

pub fn create_html_body(
    pron_type: &str,
    table_name: &str,
    stem: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<String, String> {
    let view_models =
        create_template_view_model(&pron_type, &table_name, transliterate, &exec_sql, &stem)?;
    let in_stemmed_comps_inflections = Vec::new();

    let vm = TemplateViewModel {
        table_name,
        pron_type,
        stem: &transliterate(stem)?,
        view_models,
        in_stemmed_comps_inflections,
    };

    let context = Context::from_serialize(&vm).map_err(|e| e.to_string())?;
    TEMPLATES
        .render("declension_pron_x", &context)
        .map_err(|e| e.to_string())
}

fn create_template_view_model(
    pron_type: &str,
    table_name: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
    stem: &str,
) -> Result<Vec<CaseViewModel>, String> {
    let sql = r#"
        select * from _case_values where name <> "";
        select * from _number_values where name <> "" and name <> "dual";
    "#;
    let values = exec_sql(sql)?;
    let mut view_models: Vec<CaseViewModel> = Vec::new();
    for case in values[0].iter().flatten() {
        let mut stemmed_inflections_list: Vec<Vec<String>> = Vec::new();
        for number in values[1].iter().flatten() {
            let sql = format!(
                r#"SELECT inflections FROM '{}' WHERE "case" = '{}' AND special_pron_class = '{}' AND "number" = '{}'"#,
                table_name, case, pron_type, number
            );
            let stemmed_inflections =
                inflections::get_inflections_stemmed(&sql, &exec_sql, &stem, transliterate)?;
            stemmed_inflections_list.push(stemmed_inflections);
        }

        let view_model = CaseViewModel {
            name: case.to_owned(),
            stemmed_inflections_list,
        };
        view_models.push(view_model);
    }

    println!(">>>> {:#?}", view_models);
    Ok(view_models)
}
