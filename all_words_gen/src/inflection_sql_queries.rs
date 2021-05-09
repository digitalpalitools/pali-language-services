use crate::inflection_info::InflectionsBatchInfo;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct InflectionSqlQueryBatchInfo {
    pub inflection_sql_queries: Vec<String>,
    pub inflected_forms_fetched: i64,
}

pub fn create_inflection_sql_queries(
    inflection_infos: InflectionsBatchInfo,
) -> InflectionSqlQueryBatchInfo {
    let mut sql_queries: Vec<String> = inflection_infos
        .inflection_infos
        .into_iter()
        .map(|inf| {
            format!(
                "INSERT INTO _all_words (inflection, stem_id) VALUES ('{}', {})",
                inf.inflection, inf.pali1_id
            )
        })
        .collect();

    // NOTE: Add transactions to make insertion faster https://stackoverflow.com/a/3852082/6196679
    let mut inflection_sql_queries = vec!["BEGIN".to_string()];
    inflection_sql_queries.append(&mut sql_queries);
    inflection_sql_queries.push("END".to_string());

    InflectionSqlQueryBatchInfo {
        inflection_sql_queries,
        inflected_forms_fetched: inflection_infos.inflected_forms_fetched,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inflection_info::InflectionInfo;

    #[test]
    fn test_create_inflection_infos() {
        let inflection_infos = InflectionsBatchInfo {
            inflected_forms_fetched: 10005,
            inflection_infos: vec![
                InflectionInfo {
                    pali1_id: 102,
                    inflection: "102-1".to_string(),
                },
                InflectionInfo {
                    pali1_id: 102,
                    inflection: "102-2".to_string(),
                },
                InflectionInfo {
                    pali1_id: 103,
                    inflection: "103-1".to_string(),
                },
            ],
        };

        let inflected_forms_fetched = inflection_infos.inflected_forms_fetched;
        let isqbi = create_inflection_sql_queries(inflection_infos);

        assert_eq!(inflected_forms_fetched, isqbi.inflected_forms_fetched);
        insta::assert_yaml_snapshot!(isqbi);
    }
}
