mod generators;

static HEADER_TEMPLATE: &str = r#"<header class="pls-inflection-header"><summary class="pls-inflection-summary">{{PĀLI1}} &ndash; "{{PATTERN}}" (like {{EXAMPLE_INFO}})</summary></header><br />"#;
static FOOTER_TEMPLATE: &str = r#"<footer class="pls-inflection-footer"><a class="pls-inflection-feedback-link" target="_blank" href="https://docs.google.com/forms/d/e/1FAIpQLSdqnYM0_5VeWzkFBPzyxaLqUfKWgNjI8STCpdrx4vX3hetyxw/viewform"><strong>spot a mistake? something missing? fix it here!</strong></a></footer><br />"#;

#[derive(Debug)]
pub enum InflectionClass {
    Conjugation,
    Declension,
    DeclensionPron1st,
    DeclensionPron2nd,
    DeclensionPronDual,
}

pub struct Pali1Metadata {
    pub stem: String,
    pub pattern: String,
    pub inflection_class: InflectionClass,
    pub example_info: String,
}

pub fn generate_inflection_table(
    pali1: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    let pm = get_pali1_metadata(pali1, exec_sql_structured(exec_sql))?;
    let body = generators::create_html_body(&pm, transliterate, exec_sql_structured(exec_sql))?;
    let html = append_header_footer(&pm, pali1, &body, transliterate)?;

    Ok(html)
}

fn inflection_class_from_str(ic: &str) -> InflectionClass {
    match ic {
        "verb" => InflectionClass::Conjugation,
        "pron1st" => InflectionClass::DeclensionPron1st,
        "pron2nd" => InflectionClass::DeclensionPron2nd,
        "prondual" => InflectionClass::DeclensionPronDual,
        _ => InflectionClass::Declension,
    }
}

// TODO: Pull the .to_strings out into the HOF.
// TODO: No leading/trailing spaces in _stems.pattern and _index.name.
// TODO: Negative scenarios where exec_sql does not return anything.
fn get_pali1_metadata(
    pali1: &str,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<Pali1Metadata, String> {
    let sql = format!(
        r#"select stem, pattern from '_stems' where pāli1 = "{}""#,
        pali1,
    );
    let results = exec_sql(&sql)?;
    let stem = &results[0][0][0];
    let pattern = &results[0][0][1];
    let mut pm = Pali1Metadata {
        stem: stem.clone(),
        pattern: pattern.clone(),
        inflection_class: InflectionClass::Declension,
        example_info: "".to_string(),
    };

    if !pattern.trim().is_empty() {
        let sql = format!(
            r#"select inflection_class, example_info from '_index' where name = "{}""#,
            pattern
        );
        let results = exec_sql(&sql)?;
        let inflection_class = &results[0][0][0];
        let example_info = &results[0][0][1];

        pm.inflection_class = inflection_class_from_str(inflection_class);
        pm.example_info = example_info.to_string();
    };

    Ok(pm)
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

fn exec_sql_structured<F>(f: F) -> impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>
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
