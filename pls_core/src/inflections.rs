static HEADER_TEMPLATE: &str =
    r#"<p><strong>{{PĀLI1}} &ndash; "{{PATTERN}}" ({{EXAMPLE_INFO}})</strong></p><br />"#;
static FOOTER_TEMPLATE: &str = r#"<p><a href="https://docs.google.com/forms/d/e/1FAIpQLSdqnYM0_5VeWzkFBPzyxaLqUfKWgNjI8STCpdrx4vX3hetyxw/viewform"><strong>spot a mistake? something missing? fix it here!</strong></a></p><br />"#;
static VERB_TENSE_TEMPLATE: &str = include_str!("templates/verb_tense.html");

static VERB_SQL_TEMPLATE: &str = r#"SELECT inflections FROM '{{TABLE}}' where tense = '{{TENSE}}' and person = '{{PERSON}}' and actreflx = '{{ACTREFLX}}' and "number" = '{{NUMBER}}'"#;

enum InflectionClass {
    Pron1st,
    Pron2nd,
    PronDual,
    Verb,
    Rest,
}

struct Pali1Metadata {
    stem: String,
    pattern: String,
    inflection_class: InflectionClass,
    example_info: String,
}

fn inflection_class_from_str(ic: &str) -> InflectionClass {
    match ic {
        "verb" => InflectionClass::Verb,
        "pron1st" => InflectionClass::Pron1st,
        "pron2nd" => InflectionClass::Pron2nd,
        "prondual" => InflectionClass::PronDual,
        _ => InflectionClass::Rest,
    }
}

// TODO: Pull the .to_strings out into the HOF.
// TODO: No leading/trailing spaces in _stems.pattern and _index.name.
// TODO: Negative scenarios where exec_sql does not return anything.
fn get_pali1_metadata(
    pali1: &str,
    exec_sql: impl Fn(String) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<Pali1Metadata, String> {
    let sql = format!(
        r#"select stem, pattern from '_stems' where pāli1 = "{}""#,
        pali1,
    );
    let results = exec_sql(sql)?;
    let stem = &results[0][0][0];
    let pattern = &results[0][0][1];
    let mut pm = Pali1Metadata {
        stem: stem.clone(),
        pattern: pattern.clone(),
        inflection_class: InflectionClass::Rest,
        example_info: "".to_string(),
    };

    if !pattern.trim().is_empty() {
        let sql = format!(
            r#"select inflection_class, example_info from '_index' where name = "{}""#,
            pattern
        );
        let results = exec_sql(sql)?;
        let inflection_class = &results[0][0][0];
        let example_info = &results[0][0][1];

        pm.inflection_class = inflection_class_from_str(inflection_class);
        pm.example_info = example_info.to_string();
    };

    Ok(pm)
}

// TODO: Return result, use loop.
fn get_inflections_from_table(
    table_name: &str,
    exec_sql: impl Fn(String) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Vec<String> {
    let sql = r#"
        select * from _tense_values where name <> "";
        select * from _person_values where name <> "";
        select * from _actreflx_values where name <> "";
        select * from _number_values where name <> "" and name <> "dual";
    "#;
    let results = exec_sql(sql.to_string()).unwrap();
    let mut inflections: Vec<String> = Vec::new();
    results[0].iter().flatten().for_each(|t| {
        results[1].iter().flatten().for_each(|p| {
            results[2].iter().flatten().for_each(|ar| {
                results[3].iter().flatten().for_each(|n| {
                    let sql = VERB_SQL_TEMPLATE
                        .replace("{{TABLE}}", table_name)
                        .replace("{{TENSE}}", &t)
                        .replace("{{PERSON}}", &p)
                        .replace("{{ACTREFLX}}", &ar)
                        .replace("{{NUMBER}}", &n);
                    let x = match exec_sql(sql) {
                        Ok(mut x) => {
                            if x.len() == 1 && x[0].len() == 1 && x[0][0].len() == 1 {
                                x.remove(0).remove(0).remove(0)
                            } else {
                                "".to_string()
                            }
                        }
                        Err(e) => e,
                    };
                    inflections.push(x);
                })
            })
        })
    });

    inflections
}

fn create_inflected_stems_html_fragment(stem: &str, inflections: &str) -> String {
    let html: String = inflections.split(',').fold(String::new(), |acc, e| {
        if e.is_empty() {
            acc
        } else {
            acc + &format!("{}<strong>{}</strong><br />", stem, e)
        }
    });
    html
}

fn create_html_body(
    pm: &Pali1Metadata,
    exec_sql: impl Fn(String) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> String {
    let table_name = pm.pattern.replace(" ", "_");
    match pm.inflection_class {
        InflectionClass::Verb => create_html_body_for_verb(&table_name, &pm.stem, exec_sql),
        _ => create_html_body_for_rest(&table_name, &pm.stem, exec_sql),
    }
}

fn create_html_body_for_verb(
    table_name: &str,
    stem: &str,
    exec_sql: impl Fn(String) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> String {
    let inflections = get_inflections_from_table(&table_name, exec_sql);
    let template = VERB_TENSE_TEMPLATE.to_string();
    let body: String = inflections
        .iter()
        .enumerate()
        .fold(template, |acc, (ei, e)| {
            let name = format!("|{}|", ei);
            let value = create_inflected_stems_html_fragment(stem, e);
            acc.replace(&name, &value)
        });

    body
}

fn create_html_body_for_rest(
    _table_name: &str,
    _stem: &str,
    _exec_sql: impl Fn(String) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> String {
    "<div style='color: red'>Not yet implemented!</div>".to_string()
}

fn append_header_footer(pm: &Pali1Metadata, pali1: &str, body: &str) -> String {
    let header = HEADER_TEMPLATE
        .replace("{{PĀLI1}}", pali1)
        .replace("{{PATTERN}}", &pm.pattern)
        .replace("{{EXAMPLE_INFO}}", &pm.example_info);

    format!("{}\n{}\n{}", &header, &body, FOOTER_TEMPLATE)
}

fn exec_sql_structured<F>(f: F) -> impl Fn(String) -> Result<Vec<Vec<Vec<String>>>, String>
where
    F: Fn(String) -> Result<String, String>,
{
    move |sql| {
        let result_str = f(sql)?;
        let result: Vec<Vec<Vec<String>>> =
            serde_json::from_str(&result_str).map_err(|e| e.to_string())?;
        Ok(result)
    }
}

pub fn generate_inflection_table(
    pali1: &str,
    exec_sql: fn(String) -> Result<String, String>,
    exec_sql_with_transliteration: fn(String) -> Result<String, String>,
) -> Result<String, String> {
    let pm = get_pali1_metadata(pali1, exec_sql_structured(exec_sql))?;
    let body = create_html_body(&pm, exec_sql_structured(exec_sql_with_transliteration));
    let html = append_header_footer(&pm, pali1, &body);

    Ok(html)
}
