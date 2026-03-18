use super::error::ShowMeError;
use crate::shell::config::ShellConfig;
use rusqlite::Connection;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocRow {
    pub fuente: String,
    pub texto_completo: String,
}

pub(crate) struct DocsDb;

impl DocsDb {
    /// Resuelve path desde GELI_DOCS_DB_PATH o fallback ~/.config/geliShell/docs/docs.db
    pub(crate) fn resolve_path() -> std::path::PathBuf {
        ShellConfig::assistant_docs_db_path()
    }

    /// Carga todos los registros. Error si el archivo no existe.
    pub(crate) fn load(path: &Path) -> Result<Vec<DocRow>, ShowMeError> {
        if !path.is_file() {
            return Err(ShowMeError::DbNotFound {
                path: path.to_string_lossy().into_owned(),
            });
        }

        let conn = Connection::open(path)?;

        if table_exists(&conn, "docs_metadata")? {
            return query_rows(&conn, "docs_metadata");
        }

        if table_exists(&conn, "docs")? {
            return query_rows(&conn, "docs");
        }

        Ok(Vec::new())
    }
}

fn query_rows(conn: &Connection, table: &str) -> Result<Vec<DocRow>, ShowMeError> {
    let sql = match table {
        "docs_metadata" => "SELECT fuente, texto_completo FROM docs_metadata",
        "docs" => "SELECT fuente, texto_completo FROM docs",
        _ => return Ok(Vec::new()),
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(DocRow {
            fuente: row.get::<_, String>(0)?,
            texto_completo: row.get::<_, String>(1)?,
        })
    })?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn table_exists(conn: &Connection, table: &str) -> Result<bool, ShowMeError> {
    let mut stmt = conn.prepare(
        "
        SELECT 1
        FROM sqlite_master
        WHERE type IN ('table', 'view')
          AND name = ?1
        LIMIT 1
        ",
    )?;

    let mut rows = stmt.query([table])?;
    Ok(rows.next()?.is_some())
}
