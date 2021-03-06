use serde::Serialize;
use tera::{Context, Tera};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("conjugation", include_str!("templates/conjugation.html")),
            (
                "conjugation_single_query",
                include_str!("templates/conjugation_single_query.sql"),
            ),
        ])
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
    tense_view_models: Vec<TenseViewModel>,
}

pub fn create_html_body(
    table_name: &str,
    stem: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<String, String> {
    let tense_view_models = create_template_view_model(&table_name, transliterate, &exec_sql)?;

    let vm = TemplateViewModel {
        stem: &transliterate(stem)?,
        tense_view_models,
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
                    let mut context = Context::new();
                    context.insert("table", table_name);
                    context.insert("tense", t);
                    context.insert("person", p);
                    context.insert("actreflx", ar);
                    context.insert("number", n);

                    let sql = TEMPLATES
                        .render("conjugation_single_query", &context)
                        .map_err(|e| e.to_string())?;
                    let res = match exec_sql(&sql) {
                        Ok(x) => {
                            if x.len() == 1 && x[0].len() == 1 && x[0][0].len() == 1 {
                                x[0][0][0].to_string()
                            } else {
                                "".to_string()
                            }
                        }
                        Err(e) => e,
                    };
                    let inflections: Vec<String> = res
                        .split(',')
                        .map(|s| transliterate(s).unwrap_or_else(|e| e))
                        .collect();
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
