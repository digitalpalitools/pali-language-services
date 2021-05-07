use clap::{App, Arg, ArgMatches};
use std::path::Path;

pub(crate) struct AppArgs<'a> {
    pub inflections_db_path: &'a str,
}

pub(crate) fn get_args<'a>(args: &'a ArgMatches) -> AppArgs<'a> {
    AppArgs {
        inflections_db_path: args
            .value_of("INFLECTION_DB_PATH")
            .expect("mandatory argument"),
    }
}

pub(crate) fn parse_args<'a>() -> ArgMatches<'a> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(create_inflection_db_path_arg())
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

fn validate_file_exists(s: &str) -> Result<(), String> {
    if Path::new(&s).is_file() {
        Ok(())
    } else {
        Err(format!("'{}' does not exist.", s))
    }
}
