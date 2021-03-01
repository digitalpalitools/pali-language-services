// NOTE: Ensure set window.__inflections_db before calling this function.
const execSqlCore = (sql) => {
    const results = window.__inflections_db.exec(sql)[0]
    return results.values || []
}

export const execSql = (sql) => JSON.stringify(execSqlCore(sql))

export const execSqlWithTransliteration = (sql) => JSON.stringify(execSqlCore(sql))
