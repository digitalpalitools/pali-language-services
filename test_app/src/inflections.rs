/*
TODO
o Generate inflection table
  - exe display, generate given pali1, stem, pattern
    v verbs: eti pr
    v support multiple inflections
    - verbs: -
    - verbs: *
  - sql generation work
    - place constraints on verbs / pos
    - generalize lists and order
  - test case
  - front-end display
  - non-verbs
  - remove empty columns rows
  - host determines styling
  x expand acronyms
  v additional tests: cannot have | and ,
  - simplify sql callback, publish a module that has it all
  - 3 crates - core, bin, lib
- Generate all words for a given stem
    - Generate all words for irreg
 */

static HEADER_TEMPLATE: &str =
    r#"<p><strong>{{PĀLI1}} &ndash; "{{PATTERN}}" ({{EXAMPLE_INFO}})</strong></p><br />"#;
static FOOTER_TEMPLATE: &str = r#"<p><a href="https://docs.google.com/forms/d/e/1FAIpQLSdqnYM0_5VeWzkFBPzyxaLqUfKWgNjI8STCpdrx4vX3hetyxw/viewform"><strong>spot a mistake? something missing? fix it here!</strong></a></p><br />"#;
static VERB_TENSE_TEMPLATE: &str = include_str!("templates/verb_tense.html");

static VERB_SQL_TEMPLATE: &str = r#"SELECT inflections FROM '{{TABLE}}' where tense = '{{TENSE}}' and person = '{{PERSON}}' and actreflx = '{{ACTREFLX}}' and "number" = '{{NUMBER}}'"#;
const TENSE_VALUES: &[&str] = &["pr", "imp", "opt", "fut"];
const PERSON_VALUES: &[&str] = &["3rd", "2nd", "1st"];
const ACTREFLX_VALUES: &[&str] = &["act", "reflx"];
const NUMBER_VALUES: &[&str] = &["sg", "pl"];

fn untyped_example() -> serde_json::Result<()> {
    // Some JSON input data as a &str. Maybe this comes from the user.
    let data = r#"
[["1","2"],["x","22"]]
"#;

    // Parse the string of data into serde_json::Value.
    let v: Vec<Vec<String>> = serde_json::from_str(data)?;

    // Access parts of the data by indexing with square brackets.
    println!("Please call at the number {:#?}", v[0][0]);

    Ok(())
}

fn generate_sql_queries(
    pali1: &str,
    get_table_name: fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    let x = untyped_example().unwrap();
    println!("Please call at the number {:#?}", x);
    let table = get_table_name(&pali1)?;
    let mut sqlqs: Vec<String> = Vec::new();
    TENSE_VALUES.iter().for_each(|&t| {
        PERSON_VALUES.iter().for_each(|&p| {
            ACTREFLX_VALUES.iter().for_each(|&ar| {
                NUMBER_VALUES.iter().for_each(|&n| {
                    let sqlq = VERB_SQL_TEMPLATE
                        .replace("{{TABLE}}", &table)
                        .replace("{{TENSE}}", t)
                        .replace("{{PERSON}}", p)
                        .replace("{{ACTREFLX}}", ar)
                        .replace("{{NUMBER}}", n);
                    sqlqs.push(sqlq)
                })
            })
        })
    });

    Ok(sqlqs.join("|"))
}

fn create_inflected_stems_html_fragment(stem: &str, inflections: &str) -> String {
    let html: String = inflections.split(',').fold(String::new(), |acc, e| {
        acc + &format!("{}<strong>{}</strong><br />", stem, e)
    });
    html
}

fn create_html_body(stem: &str, inflections: &str) -> String {
    let template = VERB_TENSE_TEMPLATE.to_string();
    let body: String = inflections
        .split('|')
        .enumerate()
        .fold(template, |acc, (ei, e)| {
            let name = format!("|{}|", ei);
            let value = create_inflected_stems_html_fragment(stem, e);
            acc.replace(&name, &value)
        });

    body
}

fn append_header_footer(body: &str, pali1: &str, pattern: &str, example_info: &str) -> String {
    let header = HEADER_TEMPLATE
        .replace("{{PĀLI1}}", pali1)
        .replace("{{PATTERN}}", pattern)
        .replace("{{EXAMPLE_INFO}}", example_info);

    format!("{}\n{}\n{}", &header, &body, FOOTER_TEMPLATE)
}

pub fn generate_inflection_table(
    pali1: &str,
    get_pali1_metatada: fn(&str) -> Result<String, String>,
    get_itable: fn(&str) -> Result<String, String>,
    exec_isql: fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    let metadata: Vec<String> = get_pali1_metatada(pali1)?
        .split('|')
        .map(|s| s.to_string())
        .collect();
    let sql_queries: String = generate_sql_queries(&pali1, get_itable)?;
    let inflections = exec_isql(&sql_queries)?;
    let body = create_html_body(&metadata[0], &inflections);
    let html = append_header_footer(&body, pali1, &metadata[1], &metadata[2]);

    Ok(html)
}
