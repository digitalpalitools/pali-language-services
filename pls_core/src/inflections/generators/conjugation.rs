use tera::{Context, Tera};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("conjugation", include_str!("templates/conjugation.html")),
            (
                "conjugation_tense",
                include_str!("templates/conjugation_tense.html"),
            ),
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

pub fn create_html_body(
    table_name: &str,
    stem: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<String, String> {
    let conjugation_tense_bodies =
        create_html_bodies_for_tenses(&table_name, &stem, transliterate, &exec_sql)?;

    let mut context = Context::new();
    context.insert("conjugation_tense_bodies", &conjugation_tense_bodies);

    TEMPLATES
        .render("conjugation", &context)
        .map_err(|e| e.to_string())
}

fn create_html_bodies_for_tenses(
    table_name: &str,
    stem: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<Vec<String>, String> {
    let sql = r#"
        select * from _tense_values where name <> "";
        select * from _person_values where name <> "";
        select * from _actreflx_values where name <> "";
        select * from _number_values where name <> "" and name <> "dual";
    "#;
    let values = exec_sql(sql)?;
    let mut bodies_for_tenses: Vec<String> = Vec::new();
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

        let mut context = Context::new();
        context.insert("stem", &stem);
        context.insert("tense", &t);
        context.insert("inflections_list", &inflections_list);

        let body_for_tense = TEMPLATES
            .render("conjugation_tense", &context)
            .map_err(|e| e.to_string())?;
        bodies_for_tenses.push(body_for_tense);
    }

    Ok(bodies_for_tenses)
}
