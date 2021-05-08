#[macro_use]
extern crate lazy_static;
use crate::inflections::create_inflection_infos;
use chrono::{Datelike, Utc};
use pls_core_extras::host::PlsHost;
use pls_core_extras::inflection_generator::PlsInflectionGenerator;
use pls_core_extras::logger::{ColoredConsoleLogger, PlsLogger};
use pls_core_extras::sql_access::SqlAccess;
use rusqlite::Connection;
use std::borrow::BorrowMut;
use tera::Tera;

mod args;
mod inflections;
mod stem_info;

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![(
            "batch_sql_template",
            include_str!("templates/batch_sql_template.sql"),
        )])
        .expect("Unexpected failure adding template");
        tera.autoescape_on(vec!["sql"]);
        tera
    };
}

fn main() -> Result<(), String> {
    let arg_matches = args::parse_args();
    let args = args::get_args(&arg_matches);
    print_banner();

    let connection = Connection::open(&args.inflections_db_path).map_err(|e| {
        format!(
            "Cannot open db '{}'. Error: {}.",
            args.inflections_db_path, e
        )
    })?;

    let logger = &ColoredConsoleLogger {};
    let pls_host = PlsHost {
        locale: "en",
        version: env!("CARGO_PKG_VERSION"),
        url: env!("CARGO_PKG_NAME"),
        sql_access: SqlAccess { connection },
        logger,
    };

    let igen = &PlsInflectionGenerator::new(
        "en",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_NAME"),
        args.inflections_db_path,
        logger,
    )?;

    let max_stems_to_fetch = i64::MAX;
    let max_batch_size = 100;
    let mut sii = crate::stem_info::StemInfoIterator::new(
        &pls_host.sql_access,
        max_stems_to_fetch,
        max_batch_size,
    );
    let mut ibis = sii.borrow_mut().map(|x| create_inflection_infos(x, igen));
    let mut inflections_generated = 0;
    let mut inflected_forms_fetched = 0;
    for ibi in &mut ibis {
        inflections_generated += ibi.inflection_info.len();
        inflected_forms_fetched += ibi.inflected_forms_fetched;
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
    let n = pls_host
        .sql_access
        .exec_scalar::<i32>("SELECT CAST(COUNT(*) as text) FROM '_stems'")
        .expect("");
    logger.info(&format!("_stems table rows: {}", n));

    Ok(())
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
