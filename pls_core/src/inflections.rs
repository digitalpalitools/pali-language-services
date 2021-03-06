static HEADER_TEMPLATE: &str = r#"<header class="pls-inflection-header"><summary class="pls-inflection-summary">{{PĀLI1}} &ndash; "{{PATTERN}}" (like {{EXAMPLE_INFO}})</summary></header><br />"#;
static FOOTER_TEMPLATE: &str = r#"<footer class="pls-inflection-footer"><a class="pls-inflection-feedback-link" target="_blank" href="https://docs.google.com/forms/d/e/1FAIpQLSdqnYM0_5VeWzkFBPzyxaLqUfKWgNjI8STCpdrx4vX3hetyxw/viewform"><strong>spot a mistake? something missing? fix it here!</strong></a></footer><br />"#;
static VERB_TENSE_TEMPLATE: &str = include_str!("templates/verb_tense.html");

static VERB_SQL_TEMPLATE: &str = r#"SELECT inflections FROM '{{TABLE}}' where tense = '{{TENSE}}' and person = '{{PERSON}}' and actreflx = '{{ACTREFLX}}' and "number" = '{{NUMBER}}'"#;

#[derive(Debug)]
enum InflectionClass {
    Conjugation,
    Inflection,
    InflectionPron1st,
    InflectionPron2nd,
    InflectionPronDual,
}

struct Pali1Metadata {
    stem: String,
    pattern: String,
    inflection_class: InflectionClass,
    example_info: String,
}

fn inflection_class_from_str(ic: &str) -> InflectionClass {
    match ic {
        "verb" => InflectionClass::Conjugation,
        "pron1st" => InflectionClass::InflectionPron1st,
        "pron2nd" => InflectionClass::InflectionPron2nd,
        "prondual" => InflectionClass::InflectionPronDual,
        _ => InflectionClass::Inflection,
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
        inflection_class: InflectionClass::Inflection,
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

fn get_inflections_from_table(
    table_name: &str,
    exec_sql: impl Fn(String) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<Vec<String>, String> {
    let sql = r#"
        select * from _tense_values where name <> "";
        select * from _person_values where name <> "";
        select * from _actreflx_values where name <> "";
        select * from _number_values where name <> "" and name <> "dual";
    "#;
    let values = exec_sql(sql.to_string())?;
    let mut inflections: Vec<String> = Vec::new();
    for t in values[0].iter().flatten() {
        for p in values[1].iter().flatten() {
            for ar in values[2].iter().flatten() {
                for n in values[3].iter().flatten() {
                    let sql = VERB_SQL_TEMPLATE
                        .replace("{{TABLE}}", table_name)
                        .replace("{{TENSE}}", &t)
                        .replace("{{PERSON}}", &p)
                        .replace("{{ACTREFLX}}", &ar)
                        .replace("{{NUMBER}}", &n);
                    let x = match exec_sql(sql) {
                        Ok(x) => {
                            if x.len() == 1 && x[0].len() == 1 && x[0][0].len() == 1 {
                                x[0][0][0].to_string()
                            } else {
                                "".to_string()
                            }
                        }
                        Err(e) => e,
                    };
                    inflections.push(x);
                }
            }
        }
    }

    Ok(inflections)
}

fn create_html_fragment_for_one_inflected_word(
    stem: &str,
    suffix: &str,
    transliterate: fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    Ok(format!(
        r#"<div class="pls-inflection-inflected-word">{}<span class="pls-inflection-inflected-word-suffix">{}</span></div>"#,
        transliterate(stem)?,
        transliterate(suffix)?,
    ))
}

fn create_html_fragment_for_all_inflected_words(
    stem: &str,
    inflections: &str,
    transliterate: fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    let mut html = String::new();

    for e in inflections.split(',') {
        if !e.is_empty() {
            html.push_str(&create_html_fragment_for_one_inflected_word(
                &stem,
                &e,
                transliterate,
            )?)
        }
    }

    Ok(html)
}

fn create_html_body(
    pm: &Pali1Metadata,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(String) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<String, String> {
    let table_name = pm.pattern.replace(" ", "_");
    match pm.inflection_class {
        InflectionClass::Conjugation => {
            create_html_body_for_verb(&table_name, &pm.stem, transliterate, exec_sql)
        }
        _ => Ok(create_html_body_for_rest(
            &table_name,
            &pm.stem,
            &pm.inflection_class,
            transliterate,
            exec_sql,
        )),
    }
}

fn create_html_body_for_verb(
    table_name: &str,
    stem: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(String) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<String, String> {
    let inflections = get_inflections_from_table(&table_name, exec_sql)?;
    let template = VERB_TENSE_TEMPLATE.to_string();
    let body: String = inflections
        .iter()
        .enumerate()
        .fold(template, |acc, (ei, e)| {
            let name = format!("|{}|", ei);
            // TODO: Remove unwrap.
            let value =
                create_html_fragment_for_all_inflected_words(stem, e, transliterate).unwrap();
            acc.replace(&name, &value)
        });

    Ok(body)
}

fn create_html_body_for_rest(
    _table_name: &str,
    stem: &str,
    ic: &InflectionClass,
    _transliterate: fn(&str) -> Result<String, String>,
    _exec_sql: impl Fn(String) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> String {
    format!(
        "<div style='color: red'>{} ({:#?}): Not yet implemented!</div>",
        stem, ic
    )
}

fn append_header_footer(
    pm: &Pali1Metadata,
    pali1: &str,
    body: &str,
    transliterate: fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    let header = HEADER_TEMPLATE
        .replace("{{PĀLI1}}", &transliterate(pali1)?)
        .replace("{{PATTERN}}", &pm.pattern)
        .replace("{{EXAMPLE_INFO}}", &transliterate(&pm.example_info)?);

    Ok(format!(
        r#"<div class="pls-inflection-root">{}{}{}{}{}{}{}</div>"#,
        "\n", &header, "\n", &body, "\n", FOOTER_TEMPLATE, "\n"
    ))
}

fn exec_sql_structured<F>(f: F) -> impl Fn(String) -> Result<Vec<Vec<Vec<String>>>, String>
where
    F: Fn(&str) -> Result<String, String>,
{
    move |sql| {
        let result_str = f(&sql)?;
        let result: Vec<Vec<Vec<String>>> =
            serde_json::from_str(&result_str).map_err(|e| e.to_string())?;
        Ok(result)
    }
}

pub fn generate_inflection_table(
    pali1: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    let pm = get_pali1_metadata(pali1, exec_sql_structured(exec_sql))?;
    let body = create_html_body(&pm, transliterate, exec_sql_structured(exec_sql))?;
    let html = append_header_footer(&pm, pali1, &body, transliterate)?;

    Ok(html)
}
