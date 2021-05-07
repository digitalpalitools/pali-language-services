use chrono::{Datelike, Utc};
use pls_core_extras::host::PlsHost;
use pls_core_extras::logger::ColoredConsoleLogger;
use pls_core_extras::sql_access::SqlAccess;
use rusqlite::Connection;

mod args;

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

    let pls_host = PlsHost {
        locale: "en",
        version: env!("CARGO_PKG_VERSION"),
        url: env!("CARGO_PKG_NAME"),
        sql_access: SqlAccess { connection },
        logger: &ColoredConsoleLogger {},
    };

    let html = pls_core::inflections::generate_inflection_table("no 4", true, &pls_host)?;
    println!("{:?}", html);

    let n = pls_host
        .sql_access
        .exec_scalar::<i32>("SELECT CAST(COUNT(*) as text) FROM '_stems'")
        .expect("");
    println!("{}", n);

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
