use clap::{App, Arg, ArgMatches};
use std::fmt;
use std::path::Path;

pub(crate) struct AppArgs<'a> {
    pub inflections_db_path: &'a str,
    pub max_stems_to_fetch: i64,
    pub max_batch_size: i64,
}

impl fmt::Display for AppArgs<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "... inflections_db_path: {}", self.inflections_db_path)?;
        writeln!(f, "... max_stems_to_fetch: {}", self.max_stems_to_fetch)?;
        writeln!(f, "... max_batch_size: {}", self.max_batch_size)?;
        Ok(())
    }
}

pub(crate) fn get_args<'a>(args: &'a ArgMatches) -> AppArgs<'a> {
    AppArgs {
        inflections_db_path: args
            .value_of("INFLECTION_DB_PATH")
            .expect("mandatory argument"),
        max_stems_to_fetch: args
            .value_of("MAX_STEMS_TO_FETCH")
            .expect("mandatory argument")
            .parse::<i64>()
            .expect("Valid default configured already."),
        max_batch_size: args
            .value_of("MAX_BATCH_SIZE")
            .expect("mandatory argument")
            .parse::<i64>()
            .expect("Valid default configured already."),
    }
}

pub(crate) fn parse_args<'a>() -> ArgMatches<'a> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(create_inflection_db_path_arg())
        .arg(create_max_stems_to_fetch_arg())
        .arg(create_max_batch_size_arg())
        .get_matches()
}

fn create_inflection_db_path_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("INFLECTION_DB_PATH")
        .short("i")
        .long("inflection-db")
        .value_name("INFLECTION_DB_PATH")
        .help("The path to inflections.db.")
        .required(true)
        .validator(|s| validate_file_exists(&s))
        .takes_value(true)
}

fn create_max_stems_to_fetch_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("MAX_STEMS_TO_FETCH")
        .short("s")
        .long("max-stems-to-fetch")
        .value_name("MAX_STEMS_TO_FETCH")
        .help("The maximum number of stem table entries to consider ordered by rowid.")
        .required(false)
        .validator(|s| validate_i64(&s))
        .takes_value(true)
        .default_value("9223372036854775807")
}

fn create_max_batch_size_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("MAX_BATCH_SIZE")
        .short("b")
        .long("max-batch-size")
        .value_name("MAX_BATCH_SIZE")
        .help("The maximum size of each batch of entries fetched from stem table.")
        .required(false)
        .validator(|s| validate_i64(&s))
        .takes_value(true)
        .default_value("10")
}

fn validate_file_exists(s: &str) -> Result<(), String> {
    if Path::new(&s).is_file() {
        Ok(())
    } else {
        Err(format!("'{}' does not exist.", s))
    }
}

fn validate_i64(s: &str) -> Result<(), String> {
    match s.parse::<i64>() {
        Ok(_n) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
