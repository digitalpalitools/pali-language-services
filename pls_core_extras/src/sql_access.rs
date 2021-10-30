use rusqlite::{Connection, Row, NO_PARAMS};
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

pub struct SqlAccess {
    pub connection: Connection,
}

impl SqlAccess {
    fn get_row_cells(&self, row: &Row) -> Vec<String> {
        let cells: Vec<String> = row
            .column_names()
            .iter()
            .map(|&cn| {
                let cell: String = match row.get(cn) {
                    Ok(c) => c,
                    Err(e) => e.to_string(),
                };
                cell
            })
            .collect();

        cells
    }

    pub fn exec(&self, sql: &str) -> Result<Vec<Vec<Vec<String>>>, String> {
        let mut result: Vec<Vec<Vec<String>>> = Vec::new();
        for s in sql.split(';').filter(|s| !s.trim().is_empty()) {
            let mut stmt = self.connection.prepare(s).map_err(|e| e.to_string())?;
            let mut rows = stmt.query(NO_PARAMS).map_err(|e| e.to_string())?;

            let mut table: Vec<Vec<String>> = Vec::new();
            while let Some(row) = rows.next().map_err(|e| e.to_string())? {
                table.push(self.get_row_cells(row));
            }
            result.push(table)
        }

        Ok(result)
    }

    pub fn exec_scalar<T>(&self, sql: &str) -> Result<T, String>
    where
        T: FromStr,
        T::Err: Display,
    {
        let res = self.exec(sql)?;
        if res.len() == 1 && res[0].len() == 1 && res[0][0].len() == 1 {
            res[0][0][0].parse::<T>().map_err(|e| e.to_string())
        } else {
            Err("sql query did not return a scalar".to_string())
        }
    }
}

pub fn create_sql_access() -> SqlAccess {
    let db_path = resolve_file_in_manifest_dir("inflections.db")
        .expect("must exist")
        .as_path()
        .to_str()
        .expect("must exist")
        .to_string();

    SqlAccess {
        connection: Connection::open(db_path).expect("must be valid db"),
    }
}

pub fn resolve_file_in_manifest_dir(file_name: &str) -> Result<PathBuf, String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let p1 = root.join(file_name);
    let file_path = if p1.exists() {
        p1
    } else {
        let p1 = root.parent().ok_or("???")?;
        p1.join(file_name)
    };

    Ok(file_path)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_exec_invalid_query() {
        let sa = create_sql_access();

        let ret = sa.exec("this is not a query");

        assert_eq!(ret, Err("near \"this\": syntax error".to_string()));
    }

    #[test]
    fn test_exec_does_not_return_anything() {
        let sa = create_sql_access();

        let ret = sa.exec(r#"SELECT pāli1 FROM '_stems' where pāli1 = "xxx""#);

        assert_eq!(ret, Ok(vec![vec![]]));
    }

    #[test]
    fn test_exec_sinqle_query_single_result() {
        let sa = create_sql_access();

        let ret = sa.exec(r#"SELECT pāli1 FROM '_stems' where pāli1 = "a 1""#);

        assert_eq!(ret, Ok(vec![vec![vec!["a 1".to_string()]]]));
    }

    #[test]
    fn test_exec_sinqle_query_multiple_result() {
        let sa = create_sql_access();

        let ret = sa.exec(r#"SELECT pāli1 FROM '_stems' where pāli1 like "a %""#);

        assert_eq!(
            ret,
            Ok(vec![vec![
                vec!["a 1".to_string()],
                vec!["a 2".to_string()],
                vec!["a 3".to_string()],
                vec!["a 4".to_string()],
                vec!["a 5".to_string()]
            ]])
        );
    }

    #[test]
    fn test_exec_multiple_query_single_result_each() {
        let sa = create_sql_access();

        let ret = sa.exec(r#"SELECT pāli1 FROM '_stems' where pāli1 like "a 1"; SELECT pāli1 FROM '_stems' where pāli1 like "a 2""#);

        assert_eq!(
            ret,
            Ok(vec![
                vec![vec!["a 1".to_string()]],
                vec![vec!["a 2".to_string()]]
            ])
        );
    }

    #[test]
    fn test_exec_multiple_query_multiple_result_each() {
        let sa = create_sql_access();

        let ret = sa.exec(r#"SELECT pāli1 FROM '_stems' where pāli1 like "a %"; SELECT pāli1 FROM '_stems' where pāli1 like "ababa %""#);

        assert_eq!(
            ret,
            Ok(vec![
                vec![
                    vec!["a 1".to_string()],
                    vec!["a 2".to_string()],
                    vec!["a 3".to_string()],
                    vec!["a 4".to_string()],
                    vec!["a 5".to_string()]
                ],
                vec![vec!["ababa 1".to_string()], vec!["ababa 2".to_string()]]
            ])
        );
    }

    #[test]
    fn test_exec_multiple_full() {
        let sa = create_sql_access();

        let ret = sa.exec(r#"SELECT pāli1 FROM '_stems' where pāli1 like "a %"; SELECT pāli1, stem FROM '_stems' where pāli1 like "ababa %""#);

        assert_eq!(
            ret,
            Ok(vec![
                vec![
                    vec!["a 1".to_string()],
                    vec!["a 2".to_string()],
                    vec!["a 3".to_string()],
                    vec!["a 4".to_string()],
                    vec!["a 5".to_string()]
                ],
                vec![
                    vec!["ababa 1".to_string(), "abab".to_string()],
                    vec!["ababa 2".to_string(), "abab".to_string()]
                ]
            ])
        );
    }

    #[test]
    fn test_exec_scalar_string_ok() {
        let sa = create_sql_access();

        let ret = sa.exec_scalar::<String>(r#"SELECT pāli1 FROM '_stems' where pāli1 = "a 1""#);

        assert_eq!(ret, Ok("a 1".to_string()));
    }

    #[test]
    fn test_exec_scalar_i32_ok() {
        let sa = create_sql_access();

        let ret = sa.exec_scalar::<i32>("SELECT CAST(COUNT(*) as text) FROM '_stems'");

        assert_eq!(ret, Ok(34911));
    }

    #[test]
    fn test_exec_scalar_err() {
        let sa = create_sql_access();

        let ret = sa.exec_scalar::<i32>("SELECT COUNT(*) FROM '_stems'");

        assert_eq!(ret, Err("invalid digit found in string".to_string()));
    }
}
