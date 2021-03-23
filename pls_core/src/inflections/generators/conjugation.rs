use crate::inflections;
use crate::inflections::{generators, SqlQuery};
use serde::Serialize;
use tera::{Context, Tera};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![(
            "conjugation",
            include_str!("templates/conjugation.html"),
        )])
        .expect("Unexpected failure adding template");
        tera.autoescape_on(vec!["html"]);
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
    q: &SqlQuery,
) -> Result<String, String> {
    let table_name = &generators::get_table_name_from_pattern(pattern);
    let tense_view_models = create_tense_view_models(table_name, transliterate, &q, &stem)?;
    let vm = TemplateViewModel {
        stem,
        view_models: tense_view_models,
    };
    let context = Context::from_serialize(&vm).map_err(|e| e.to_string())?;
    TEMPLATES
        .render("conjugation", &context)
        .map_err(|e| e.to_string())
}

struct ParameterValues {
    pub t_values: Vec<String>,
    pub p_values: Vec<String>,
    pub ar_values: Vec<String>,
    pub n_values: Vec<String>,
}

fn query_parameter_values(q: &SqlQuery) -> Result<ParameterValues, String> {
    let sql = r#"
        select * from _tense_values where name <> "";
        select * from _person_values where name <> "";
        select * from _actreflx_values where name <> "";
        select * from _number_values where name <> "" and name <> "dual";
    "#;

    let values = q.exec(sql)?;
    Ok(ParameterValues {
        t_values: values[0].to_owned().into_iter().flatten().collect(),
        p_values: values[1].to_owned().into_iter().flatten().collect(),
        ar_values: values[2].to_owned().into_iter().flatten().collect(),
        n_values: values[3].to_owned().into_iter().flatten().collect(),
    })
}

fn create_tense_view_models(
    table_name: &str,
    transliterate: fn(&str) -> Result<String, String>,
    q: &SqlQuery,
    stem: &str,
) -> Result<Vec<TenseViewModel>, String> {
    let pvs = query_parameter_values(&q)?;

    let mut view_models: Vec<TenseViewModel> = Vec::new();
    for t in &pvs.t_values {
        if inflections::query_has_no_results(
            &format!(
                r#"select cast(count(*) as text) from {} where tense = "{}""#,
                table_name, t
            ),
            &q,
        )? {
            continue;
        }

        let mut ar_values_exist: Vec<bool> = Vec::new();
        for ar in &pvs.ar_values {
            ar_values_exist.push(!inflections::query_has_no_results(
                &format!(r#"select cast(count(*) as text) from '{}' where tense = "{}" and actreflx = "{}""#, table_name, t, ar),
                &q,
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
                    let inflections = inflections::get_inflections(&stem, &sql, transliterate, &q);
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
