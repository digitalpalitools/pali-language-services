static VERB_TENSE_TEMPLATE: &str = include_str!("templates/verb_tense.html");
static VERB_SQL_TEMPLATE: &str = r#"SELECT inflections FROM '{{TABLE}}' where tense = '{{TENSE}}' and person = '{{PERSON}}' and actreflx = '{{ACTREFLX}}' and "number" = '{{NUMBER}}'"#;

pub fn create_html_body(
    table_name: &str,
    stem: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
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

fn get_inflections_from_table(
    table_name: &str,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<Vec<String>, String> {
    let sql = r#"
        select * from _tense_values where name <> "";
        select * from _person_values where name <> "";
        select * from _actreflx_values where name <> "";
        select * from _number_values where name <> "" and name <> "dual";
    "#;
    let values = exec_sql(sql)?;
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
                    let x = match exec_sql(&sql) {
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
