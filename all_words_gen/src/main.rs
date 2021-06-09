use crate::inflection_info::create_inflection_infos;
use crate::inflection_sql_queries::create_inflection_sql_queries;
use chrono::{Datelike, Utc};
use pls_core_extras::inflection_generator::PlsInflectionGenerator;
use pls_core_extras::logger::{ColoredConsoleLogger, PlsLogger};
use std::borrow::BorrowMut;

mod args;
mod inflection_info;
mod inflection_sql_queries;
mod stem_info;

fn main() -> Result<(), String> {
    let arg_matches = args::parse_args();
    let args = args::get_args(&arg_matches);
    print_banner();

    let logger = &ColoredConsoleLogger {};
    logger.info("Generating all words with the following parameters:");
    logger.info(&format!(
        "... inflections_db_path: {}",
        args.inflections_db_path
    ));
    logger.info(&format!(
        "... max_stems_to_fetch: {}",
        args.max_stems_to_fetch
    ));
    logger.info(&format!("... max_batch_size: {}", args.max_batch_size));
    logger.info("");

    let igen = &PlsInflectionGenerator::new(
        "en",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_NAME"),
        args.inflections_db_path,
        logger,
    )?;

    logger.info("(Re)Creating _all_words table...");
    create_all_words_table(&igen)?;

    logger.info("Inserting inflections into _all_words...");
    let mut sii = crate::stem_info::StemInfoIterator::new(
        &igen.inflection_host.sql_access,
        args.max_stems_to_fetch,
        args.max_batch_size,
    );
    let mut ibis = sii
        .borrow_mut()
        .map(|x| create_inflection_infos(x, igen))
        .map(create_inflection_sql_queries);
    let mut inflections_generated = 0;
    let mut inflected_forms_fetched = 0;
    let mut n = 0;
    for ibi in &mut ibis {
        inflections_generated += ibi.inflection_sql_queries.len() - 2;
        inflected_forms_fetched += ibi.inflected_forms_fetched;

        let batch_query = ibi.inflection_sql_queries.join(";\n");
        match &igen.inflection_host.sql_access.exec(&batch_query) {
            Ok(_) => Ok(()),
            Err(e) => {
                logger.error(&format!("Insertion into db failed with {}", e));
                Err(e)
            }
        }?;

        n += 1;
        if n % 1000 == 0 {
            logger.info(&format!(
                "... inserted {:05} entries into db. ({}).",
                n,
                ibi.inflection_sql_queries[ibi.inflection_sql_queries.len() - 2]
            ));
        }
    }

    logger.info("Summary:");
    logger.info(&format!("... Head words fetched: {}", sii.stems_fetched));
    logger.info(&format!(
        "... Head word batches fetched: {}",
        sii.batches_fetched
    ));
    logger.info(&format!(
        "... Inflection generated: {}",
        inflections_generated
    ));
    logger.info(&format!(
        "... Inflected forms fetched: {}",
        inflected_forms_fetched
    ));
    logger.info(&format!("... Error: {:?}", sii.error));

    logger.info("");
    print_table_word_count("_stems", igen);
    print_table_word_count("_all_words", igen);

    Ok(())
}

fn create_all_words_table(igen: &PlsInflectionGenerator) -> Result<(), String> {
    let query = r#"DROP TABLE IF EXISTS _all_words; CREATE TABLE _all_words (inflectionIndex INTEGER PRIMARY KEY, inflection TEXT NOT NULL, stem_id INTEGER NOT NULL);"#;
    match igen.inflection_host.sql_access.exec(query) {
        Ok(_) => Ok(()),
        Err(e) => {
            igen.inflection_host
                .logger
                .error(&format!("Insertion into db failed with {}", e));
            Err(e)
        }
    }
}

fn print_banner() {
    println!(
        "{} - {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_DESCRIPTION")
    );
    println!(
        "(c) 2020 - {}, {}",
        Utc::now().year(),
        env!("CARGO_PKG_AUTHORS")
    );
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("This work is licensed under the {} license (https://creativecommons.org/licenses/by-nc-sa/4.0/)", env!("CARGO_PKG_LICENSE"));
    println!();
}

fn print_table_word_count(table_name: &str, igen: &PlsInflectionGenerator) {
    let n = igen
        .inflection_host
        .sql_access
        .exec_scalar::<i32>(&format!(
            "SELECT CAST(COUNT(*) as text) FROM '{}'",
            table_name
        ))
        .expect("");
    igen.inflection_host
        .logger
        .info(&format!("Total {} table rows: {}", table_name, n));
}
