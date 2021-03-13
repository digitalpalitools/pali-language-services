use crate::inflections;
use crate::inflections::generators;
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
    ar_values_exist: Vec<bool>,
}

#[derive(Serialize)]
struct TemplateViewModel<'a> {
    stem: &'a str,
    view_models: Vec<TenseViewModel>,
}

pub fn create_html_body(
    pattern: &str,
    stem: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<String, String> {
    let table_name = &generators::get_table_name_from_pattern(pattern);
    let tense_view_models = create_tense_view_models(table_name, transliterate, &exec_sql, &stem)?;
    let vm = TemplateViewModel {
        stem,
        view_models: tense_view_models,
    };
    let context = Context::from_serialize(&vm).map_err(|e| e.to_string())?;
    TEMPLATES
        .render("conjugation", &context)
        .map_err(|e| e.to_string())
}

fn query_has_no_results(
    query: &str,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<bool, String> {
    let count = &exec_sql(&query)?[0][0][0];
    Ok(count.eq("0"))
}

struct ParameterValues {
    pub t_values: Vec<String>,
    pub p_values: Vec<String>,
    pub ar_values: Vec<String>,
    pub n_values: Vec<String>,
}

fn query_parameter_values(
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<ParameterValues, String> {
    let sql = r#"
        select * from _tense_values where name <> "";
        select * from _person_values where name <> "";
        select * from _actreflx_values where name <> "";
        select * from _number_values where name <> "" and name <> "dual";
    "#;

    let mut values = exec_sql(sql)?;
    Ok(ParameterValues {
        t_values: values.remove(0).into_iter().flatten().collect(),
        p_values: values.remove(0).into_iter().flatten().collect(),
        ar_values: values.remove(0).into_iter().flatten().collect(),
        n_values: values.remove(0).into_iter().flatten().collect(),
    })
}

fn create_tense_view_models(
    table_name: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
    stem: &str,
) -> Result<Vec<TenseViewModel>, String> {
    let pvs = query_parameter_values(&exec_sql)?;

    let mut view_models: Vec<TenseViewModel> = Vec::new();
    for t in &pvs.t_values {
        if query_has_no_results(
            &format!(
                r#"select cast(count(*) as text) from {} where tense = "{}""#,
                table_name, t
            ),
            &exec_sql,
        )? {
            continue;
        }

        let mut ar_values_exist: Vec<bool> = Vec::new();
        for ar in &pvs.ar_values {
            ar_values_exist.push(!query_has_no_results(
                &format!(r#"select cast(count(*) as text) from '{}' where tense = "{}" and actreflx = "{}""#, table_name, t, ar),
                &exec_sql,
            )?);
        }

        let mut inflections_list: Vec<Vec<String>> = Vec::new();
        for p in &pvs.p_values {
            for ar in &pvs.ar_values {
                for n in &pvs.n_values {
                    let sql = format!(
                        r#"SELECT inflections FROM '{}' WHERE tense = '{}' AND person = '{}' AND actreflx = '{}' AND "number" = '{}'"#,
                        table_name, t, p, ar, n,
                    );
                    let inflections =
                        inflections::get_inflections(&stem, &sql, transliterate, &exec_sql);
                    inflections_list.push(inflections);
                }
            }
        }

        let view_model = TenseViewModel {
            name: t.to_owned(),
            inflections_list,
            ar_values_exist,
        };
        view_models.push(view_model);
    }

    Ok(view_models)
}
