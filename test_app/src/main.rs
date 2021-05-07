use pls_core_extras::host::PlsHost;
use pls_core_extras::logger::ColoredConsoleLogger;
use pls_core_extras::sql_access::SqlAccess;
use rusqlite::Connection;
use std::path::PathBuf;

fn main() -> Result<(), String> {
    println!("{:?}", pls_core::alphabet::PALI_ALPHABET_ROMAN);
    let x = pls_core::alphabet::PaliAlphabet::Aa;
    println!("ā > bh? {:#?}", x > pls_core::alphabet::PaliAlphabet::Bh);

    let inflections_db_path = resolve_file_in_manifest_dir("inflections.db")
        .expect("must exist")
        .as_path()
        .to_str()
        .expect("must exist")
        .to_owned();
    let connection = Connection::open(&inflections_db_path)
        .map_err(|e| format!("Cannot open db '{}'. Error: {}.", &inflections_db_path, e))?;

    let pls_host = PlsHost {
        locale: "en",
        version: "host version v0.0.1",
        url: "the table is hosted here",
        sql_access: SqlAccess { connection },
        logger: &ColoredConsoleLogger {},
    };

    let html = pls_core::inflections::generate_all_inflections("ababa 1", &pls_host)?;
    println!("{:#?}", html);

    Ok(())
}

fn resolve_file_in_manifest_dir(file_name: &str) -> Result<PathBuf, String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let p1 = root.join(file_name);
    let file_path = if p1.exists() {
        p1
    } else {
        let p1 = root.parent().ok_or("")?;
        p1.join(file_name)
    };

    Ok(file_path)
}
