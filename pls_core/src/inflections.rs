static HEADER_TEMPLATE: &str =
    r#"<p><strong>{{PĀLI1}} &ndash; "{{PATTERN}}" (like {{EXAMPLE_INFO}})</strong></p><br />"#;
static FOOTER_TEMPLATE: &str = r#"<p><a target="_blank" href="https://docs.google.com/forms/d/e/1FAIpQLSdqnYM0_5VeWzkFBPzyxaLqUfKWgNjI8STCpdrx4vX3hetyxw/viewform"><strong>spot a mistake? something missing? fix it here!</strong></a></p><br />"#;
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

// TODO: Return result, use loop.
// TODO: Remove all unwraps.
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
    let values = exec_sql(sql.to_string()).unwrap();
    let mut inflections: Vec<String> = Vec::new();
    values[0].iter().flatten().for_each(|t| {
        values[1].iter().flatten().for_each(|p| {
            values[2].iter().flatten().for_each(|ar| {
                values[3].iter().flatten().for_each(|n| {
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

// TODO: Remove unwrap.
fn create_inflected_stems_html_fragment(
    stem: &str,
    inflections: &str,
    transliterate: fn(&str) -> Result<String, String>,
) -> String {
    let html: String = inflections.split(',').fold(String::new(), |acc, e| {
        if e.is_empty() {
            acc
        } else {
            acc + &format!(
                "{}<strong>{}</strong><br />",
                transliterate(stem).unwrap(),
                transliterate(e).unwrap(),
            )
        }
    });

    html
}

fn create_html_body(
    pm: &Pali1Metadata,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(String) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> String {
    let table_name = pm.pattern.replace(" ", "_");
    match pm.inflection_class {
        InflectionClass::Conjugation => {
            create_html_body_for_verb(&table_name, &pm.stem, transliterate, exec_sql)
        }
        _ => create_html_body_for_rest(
            &table_name,
            &pm.stem,
            &pm.inflection_class,
            transliterate,
            exec_sql,
        ),
    }
}

fn create_html_body_for_verb(
    table_name: &str,
    stem: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(String) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> String {
    let inflections = get_inflections_from_table(&table_name, exec_sql);
    let template = VERB_TENSE_TEMPLATE.to_string();
    let body: String = inflections
        .iter()
        .enumerate()
        .fold(template, |acc, (ei, e)| {
            let name = format!("|{}|", ei);
            let value = create_inflected_stems_html_fragment(stem, e, transliterate);
            acc.replace(&name, &value)
        });

    body
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

    Ok(format!("{}\n{}\n{}", &header, &body, FOOTER_TEMPLATE))
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

// TODO: JS callbacks should have &str as parameter
pub fn generate_inflection_table(
    pali1: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: fn(String) -> Result<String, String>,
) -> Result<String, String> {
    let pm = get_pali1_metadata(pali1, exec_sql_structured(exec_sql))?;
    let body = create_html_body(&pm, transliterate, exec_sql_structured(exec_sql));
    let html = append_header_footer(&pm, pali1, &body, transliterate)?;

    Ok(html)
}

fn get_inflections_for_pattern(
    pattern: &str,
    exec_sql: impl Fn(String) -> Result<Vec<Vec<String>>, String>,
) -> Result<Vec<Vec<String>>, String> {
    exec_sql("Select * from {}".replace("{}", pattern))
}

fn get_words_for_indeclinable_stem(paliword: &str) -> Vec<(String, String, String, String)> {
    vec![(
        paliword.chars().filter(|c| !c.is_digit(10)).collect(),
        paliword.to_string(),
        " ".to_string(),
        "ind".to_string(),
    )]
}
fn get_words_for_irregular_stem(
    paliword: &str,
    pattern: &str,
    _exec_sql: fn(String) -> Result<String, String>,
) -> Vec<(String, String, String, String)> {
    let inflections: Vec<Vec<String>> =
        get_inflections_for_pattern(pattern, exec_sql_structured(_exec_sql)).unwrap();
    let mut inflected_words_irregular_stem: Vec<(String, String, String, String)> = Vec::new();
    for mut i in inflections {
        for j in i.pop().unwrap().split(",") {
            inflected_words_irregular_stem.push((
                j.to_string(),
                paliword.to_string(),
                i.join(" ").to_string(),
                "*".to_string(),
            ))
        }
    }
    inflected_words_irregular_stem
}
fn get_words_for_regular_stem(
    paliword: &str,
    stem: &str,
    pattern: &str,
    _exec_sql: fn(String) -> Result<String, String>,
) -> Vec<(String, String, String, String)> {
    let mut inflected_words_regular_stem: Vec<(String, String, String, String)> = Vec::new();
    let inflections: Vec<Vec<String>> =
        get_inflections_for_pattern(pattern, exec_sql_structured(_exec_sql)).unwrap();
    for mut i in inflections {
        for j in i.pop().unwrap().split(",") {
            inflected_words_regular_stem.push((
                [stem, j].join("").to_string(),
                paliword.to_string(),
                i.join(" ").to_string(),
                " ".to_string(),
            ))
        }
    }
    inflected_words_regular_stem
}

pub fn generate_all_inflected_words(
    paliword: &str,
    stem: &str,
    pattern: &str,
    _exec_sql_notransliteration: fn(String) -> Result<String, String>,
    _exec_sql_transliterated: fn(String) -> Result<String, String>,
) -> Result<Vec<(String, String, String, String)>, String> {
    let inflected_words: Vec<(String, String, String, String)> = match stem {
        "-" => get_words_for_indeclinable_stem(paliword),
        "*" => get_words_for_irregular_stem(paliword, pattern, _exec_sql_notransliteration),
        _ => get_words_for_regular_stem(paliword, stem, pattern, _exec_sql_notransliteration),
    };
    Ok(inflected_words)
}
