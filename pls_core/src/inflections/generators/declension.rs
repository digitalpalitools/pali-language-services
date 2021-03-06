use serde::Serialize;
use tera::{Context, Tera};
// use std::fs;

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("declension", include_str!("templates/declension.html")),
            (
                "declension_single_query",
                include_str!("templates/declension_single_query.sql"),
            ),
        ])
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
    table_name: &'a str,
    stem: &'a str,
    case_view_models: Vec<CaseViewModel>,
    in_comps: &'a str,
}

pub fn create_html_body(
    table_name: &str,
    stem: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<String, String> {
    let case_view_models = create_template_view_model(&table_name, transliterate, &exec_sql)?;

    let mut context = Context::new();
    context.insert("table", table_name);
    context.insert("case", "");
    context.insert("gender", "");
    context.insert("number", "");

    let sql = TEMPLATES
        .render("declension_single_query", &context)
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

    let vm = TemplateViewModel {
        table_name,
        stem: &transliterate(stem)?,
        case_view_models,
        in_comps: &transliterate(&res)?,
    };

    let context = Context::from_serialize(&vm).map_err(|e| e.to_string())?;
    let xxx = TEMPLATES
        .render("declension", &context)
        .map_err(|e| e.to_string())?;
    // fs::write("d:/delme/declension.txt", &xxx);

    Ok(xxx)
}

fn create_template_view_model(
    table_name: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<Vec<CaseViewModel>, String> {
    let sql = r#"
        select * from _case_values where name <> "";
        select * from _gender_values where name <> "" and name <> "x";
        select * from _number_values where name <> "" and name <> "dual";
    "#;
    let values = exec_sql(sql)?;
    let mut view_models: Vec<CaseViewModel> = Vec::new();
    for case in values[0].iter().flatten() {
        let mut inflections_list: Vec<Vec<String>> = Vec::new();
        for gender in values[1].iter().flatten() {
            for number in values[2].iter().flatten() {
                let mut context = Context::new();
                context.insert("table", table_name);
                context.insert("case", case);
                context.insert("gender", gender);
                context.insert("number", number);

                let sql = TEMPLATES
                    .render("declension_single_query", &context)
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

        let view_model = CaseViewModel {
            name: case.to_owned(),
            inflections_list,
        };
        view_models.push(view_model);
    }

    Ok(view_models)
}
