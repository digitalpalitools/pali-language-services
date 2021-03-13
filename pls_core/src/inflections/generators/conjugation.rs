use crate::inflections;
use serde::Serialize;
use tera::{Context, Tera};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![(
            "conjugation",
            include_str!("templates/conjugation.html"),
        )])
        .unwrap();
        tera.autoescape_on(vec!["html", ".sql"]);
        tera
    };
}

#[derive(Serialize)]
struct TenseViewModel {
    name: String,
    inflections_list: Vec<Vec<String>>,
}

#[derive(Serialize)]
struct TemplateViewModel<'a> {
    stem: &'a str,
    view_models: Vec<TenseViewModel>,
}

pub fn create_html_body(
    table_name: &str,
    stem: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<String, String> {
    let tense_view_models =
        create_template_view_model(&table_name, transliterate, &exec_sql, &stem)?;
    let vm = TemplateViewModel {
        stem,
        view_models: tense_view_models,
    };
    let context = Context::from_serialize(&vm).map_err(|e| e.to_string())?;
    TEMPLATES
        .render("conjugation", &context)
        .map_err(|e| e.to_string())
}

fn create_template_view_model(
    table_name: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
    stem: &str,
) -> Result<Vec<TenseViewModel>, String> {
    let sql = r#"
        select * from _tense_values where name <> "";
        select * from _person_values where name <> "";
        select * from _actreflx_values where name <> "";
        select * from _number_values where name <> "" and name <> "dual";
    "#;
    let values = exec_sql(sql)?;
    let mut view_models: Vec<TenseViewModel> = Vec::new();
    for t in values[0].iter().flatten() {
        let count_sql = format!(
            r#"select cast(count(*) as text) from {} where tense = "{}""#,
            table_name, t
        );
        let count = &exec_sql(&count_sql)?[0][0][0];
        if count.eq("0") {
            continue;
        }

        let mut inflections_list: Vec<Vec<String>> = Vec::new();
        for p in values[1].iter().flatten() {
            for ar in values[2].iter().flatten() {
                for n in values[3].iter().flatten() {
                    let sql = format!(
                        r#"SELECT inflections FROM '{}' WHERE tense = '{}' AND person = '{}' AND actreflx = '{}' AND "number" = '{}'"#,
                        table_name, t, p, ar, n,
                    );
                    let inflections = inflections::get_inflections(
                        &stem,
                        &sql,
                        transliterate,
                        &exec_sql,
                    );
                    inflections_list.push(inflections);
                }
            }
        }

        let view_model = TenseViewModel {
            name: t.to_owned(),
            inflections_list,
        };
        view_models.push(view_model);
    }

    Ok(view_models)
}
