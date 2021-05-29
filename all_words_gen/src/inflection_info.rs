use crate::stem_info::StemInfo;
use pls_core_extras::inflection_generator::InflectionGenerator;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct InflectionInfo {
    pub pali1_id: i64,
    pub inflection: String,
}

#[derive(Serialize, Debug)]
pub struct InflectionsBatchInfo {
    pub inflection_infos: Vec<InflectionInfo>,
    pub inflected_forms_fetched: i64,
}

pub fn create_inflection_infos(
    stem_infos: Vec<StemInfo>,
    igen: &dyn InflectionGenerator,
) -> InflectionsBatchInfo {
    let mut ibi = InflectionsBatchInfo {
        inflection_infos: vec![],
        inflected_forms_fetched: 0,
    };

    for stem_info in stem_infos {
        let mut infs = igen.generate_all_inflections(&stem_info.pali1);
        infs.sort();
        infs.dedup();
        if infs.is_empty() {
            ibi.inflected_forms_fetched += 1;
        }

        let mut inf_infos: Vec<InflectionInfo> = infs
            .into_iter()
            .map(|inf| InflectionInfo {
                pali1_id: stem_info.row_id,
                inflection: inf,
            })
            .collect();
        ibi.inflection_infos.append(&mut inf_infos);
    }

    ibi
}

#[cfg(test)]
mod tests {
    use super::*;
    use pls_core_extras::inflection_generator::PlsInflectionGenerator;
    use pls_core_extras::logger::{NullLogger, PlsLogger};
    use pls_core_extras::sql_access::resolve_file_in_manifest_dir;

    #[test]
    fn test_create_inflection_infos() {
        let db_path = &get_db_path();
        let igen = create_inflection_generator(db_path, &NullLogger {});
        let stems = vec![
            StemInfo {
                row_id: 7,
                pali1: "ababa 1".to_string(),
            },
            StemInfo {
                row_id: 8,
                pali1: "ababa 2".to_string(),
            },
            StemInfo {
                row_id: 32,
                pali1: "abbahe".to_string(),
            },
        ];
        let infl_infos = create_inflection_infos(stems, &igen);

        insta::assert_yaml_snapshot!(infl_infos);
    }

    fn create_inflection_generator<'a>(
        db_path: &'a str,
        logger: &'a dyn PlsLogger,
    ) -> PlsInflectionGenerator<'a> {
        let igen = PlsInflectionGenerator::new(
            "en",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_NAME"),
            db_path,
            logger,
        )
        .expect("unexpected test setup failure");

        igen
    }

    fn get_db_path() -> String {
        resolve_file_in_manifest_dir("inflections.db")
            .expect("must exist")
            .as_path()
            .to_str()
            .expect("must exist")
            .to_string()
    }
}
