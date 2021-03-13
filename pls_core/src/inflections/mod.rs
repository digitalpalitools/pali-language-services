mod generators;

use tera::{Context, Tera};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![("output", include_str!("templates/output.html"))])
            .unwrap();
        tera.autoescape_on(vec!["html", ".sql"]);
        tera
    };
}

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
    pub like: String,
}

pub fn generate_inflection_table(
    pali1: &str,
    host_url: &str,
    host_version: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    let pm = get_pali1_metadata(pali1, exec_sql_structured(exec_sql))?;
    let body = generators::create_html_body(&pm, transliterate, exec_sql_structured(exec_sql))?;

    generate_output(&pm, pali1, host_url, host_version, &body, transliterate)
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

// TODO: No leading/trailing spaces in _stems.pattern and _index.name.
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
        stem: if !stem.eq("*") {
            stem.clone()
        } else {
            "".to_string()
        },
        pattern: pattern.clone(),
        inflection_class: InflectionClass::Declension,
        like: "".to_string(),
    };

    if !pattern.trim().is_empty() {
        let sql = format!(
            r#"select inflection_class, like from '_index' where name = "{}""#,
            pattern
        );
        let results = exec_sql(&sql)?;
        let inflection_class = &results[0][0][0];
        let like = &results[0][0][1];

        pm.inflection_class = inflection_class_from_str(inflection_class);
        pm.like = like.to_string();
    };

    Ok(pm)
}

fn generate_output(
    pm: &Pali1Metadata,
    pali1: &str,
    host_url: &str,
    host_version: &str,
    body: &str,
    transliterate: fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    let feedback_form_url = match pm.inflection_class {
        InflectionClass::Conjugation => {
            "https://docs.google.com/forms/d/e/1FAIpQLSeJpx7TsISkYEXzxvbBtOH25T-ZO1Z5NFdujO5SD9qcAH_i1A/viewform"
        }
        _ => { // All declensions.
            "https://docs.google.com/forms/d/e/1FAIpQLSeoxZiqvIWadaLeuXF4f44NCqEn49-B8KNbSvNer5jxgRYdtQ/viewform"
        }
    };

    let mut context = Context::new();
    context.insert("pali1", &transliterate(pali1)?);
    context.insert("pattern", &pm.pattern);
    context.insert("like", &transliterate(&pm.like)?);
    context.insert("body", &body);
    context.insert("feedback_form_url", &feedback_form_url);
    context.insert("host_url", &host_url);
    context.insert("host_version", &host_version);

    TEMPLATES
        .render("output", &context)
        .map_err(|e| e.to_string())
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

#[derive(Debug, Clone)]
pub struct InflectedWordMetadata {
    pub inflected_word: String,
    pub stem_word: String,
    pub grammar: String,
    pub comment: String,
}

impl InflectedWordMetadata {
    pub fn simple_representation(self) -> (String, String, String, String) {
        (
            self.inflected_word,
            self.stem_word,
            self.grammar,
            self.comment,
        )
    }
}

fn get_inflections_for_pattern(
    pattern: &str,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Result<Vec<Vec<Vec<String>>>, String> {
    exec_sql(&format!("Select * from {}", pattern))
}

fn get_words_for_indeclinable_stem(paliword: &str) -> Vec<InflectedWordMetadata> {
    vec![InflectedWordMetadata {
        inflected_word: paliword.chars().filter(|c| !c.is_digit(10)).collect(),
        stem_word: paliword.to_string(),
        grammar: " ".to_string(),
        comment: "ind".to_string(),
    }]
}

fn get_words_for_irregular_stem(
    paliword: &str,
    pattern: &str,
    _exec_sql: fn(&str) -> Result<String, String>,
) -> Result<Vec<InflectedWordMetadata>, String> {
    let inflections: Vec<Vec<String>> =
        get_inflections_for_pattern(pattern, exec_sql_structured(_exec_sql))?
            .pop()
            .ok_or_else(|| format!("No pattern found for {}", pattern))?;
    let mut inflected_words_irregular_stem: Vec<InflectedWordMetadata> = Vec::new();
    for mut inflection_row in inflections {
        for inflection in inflection_row
            .pop()
            .ok_or_else(|| format!("No pattern found for {}", pattern))?
            .split(',')
        {
            inflected_words_irregular_stem.push(InflectedWordMetadata {
                inflected_word: inflection.to_string(),
                stem_word: paliword.to_string(),
                grammar: inflection_row.join(" ").to_string(),
                comment: "*".to_string(),
            })
        }
    }
    Ok(inflected_words_irregular_stem)
}

fn get_words_for_regular_stem(
    paliword: &str,
    stem: &str,
    pattern: &str,
    _exec_sql: fn(&str) -> Result<String, String>,
) -> Result<Vec<InflectedWordMetadata>, String> {
    let mut inflected_words_regular_stem: Vec<InflectedWordMetadata> = Vec::new();
    let inflections: Vec<Vec<String>> =
        get_inflections_for_pattern(pattern, exec_sql_structured(_exec_sql))?
            .pop()
            .ok_or_else(|| format!("No pattern found for {}", pattern))?;
    for mut inflection_row in inflections {
        for inflection in inflection_row
            .pop()
            .ok_or_else(|| format!("No pattern found for {}", pattern))?
            .split(',')
        {
            inflected_words_regular_stem.push(InflectedWordMetadata {
                inflected_word: [stem, inflection].join("").to_string(),
                stem_word: paliword.to_string(),
                grammar: inflection_row.join(" ").to_string(),
                comment: " ".to_string(),
            })
        }
    }
    Ok(inflected_words_regular_stem)
}

pub fn generate_all_inflected_words(
    paliword: &str,
    stem: &str,
    pattern: &str,
    _exec_sql: fn(&str) -> Result<String, String>,
) -> Result<Vec<InflectedWordMetadata>, String> {
    let inflected_words: Vec<InflectedWordMetadata> = match stem {
        "-" => get_words_for_indeclinable_stem(paliword),
        "*" => get_words_for_irregular_stem(paliword, pattern, _exec_sql)?,
        _ => get_words_for_regular_stem(paliword, stem, pattern, _exec_sql)?,
    };
    Ok(inflected_words)
}

fn join_and_transliterate_if_not_empty(
    stem: &str,
    suffix: &str,
    transliterate: fn(&str) -> Result<String, String>,
) -> String {
    if suffix.is_empty() {
        "".to_string()
    } else {
        transliterate(&format!("{}{}", stem, suffix)).unwrap_or_else(|e| e)
    }
}

fn get_inflections(
    stem: &str,
    sql: &str,
    transliterate: fn(&str) -> Result<String, String>,
    exec_sql: impl Fn(&str) -> Result<Vec<Vec<Vec<String>>>, String>,
) -> Vec<String> {
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
        .map(|s| join_and_transliterate_if_not_empty(stem, s, transliterate))
        .collect();

    inflections
}
