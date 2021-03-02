const execSqlCore = (sql) => {
    if (window.__pali_language_services_execSqlCore) {
        return window.__pali_language_services_execSqlCore(sql)
    }

    const results = window.__pali_language_services_inflections_db.exec(sql)[0] || { values: [[]] }
    return results.values
}

export const execSql = (sql) => {
    try {
        return JSON.stringify(execSqlCore(sql))
    } catch (e) {
        console.error('pali-language-services-dal.execSql', e)
        throw e
    }
}

export const execSqlWithTransliteration = (sql) => {
    try {
        const table = execSqlCore(sql).map(row => row.map(cell => window.__pali_script_converter_convert(cell.toString())))
        return JSON.stringify(table)
    } catch (e) {
        console.error('pali-language-services-dal.execSqlWithTransliteration', e)
        throw e
    }
}
