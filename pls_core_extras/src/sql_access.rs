use rusqlite::{Connection, Row, NO_PARAMS};

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

    pub fn exec_sql_core(&self, sql: &str) -> rusqlite::Result<Vec<Vec<Vec<String>>>, String> {
        let mut result: Vec<Vec<Vec<String>>> = Vec::new();
        for s in sql.split(';').filter(|s| !s.trim().is_empty()) {
            let mut stmt = self.connection.prepare(&s).map_err(|e| e.to_string())?;
            let mut rows = stmt.query(NO_PARAMS).map_err(|e| e.to_string())?;

            let mut table: Vec<Vec<String>> = Vec::new();
            while let Some(row) = rows.next().map_err(|e| e.to_string())? {
                table.push(self.get_row_cells(row));
            }
            result.push(table)
        }

        Ok(result)
    }
}
