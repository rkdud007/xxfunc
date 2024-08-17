use std::path::Path;

use eyre::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

// database for storing and retrieving wasm modules
pub struct ModuleDatabase {
    pool: Pool<SqliteConnectionManager>,
}

impl ModuleDatabase {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::new(manager)?;

        pool.get().unwrap().execute(
            "CREATE TABLE IF NOT EXISTS modules (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                binary BLOB NOT NULL
            )",
            [],
        )?;

        Ok(ModuleDatabase { pool })
    }

    pub fn insert(&self, name: &str, binary: &[u8]) -> Result<()> {
        self.pool
            .get()
            .unwrap()
            .execute("INSERT INTO modules (name, binary) VALUES (?1, ?2)", params![name, binary])?;
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<Option<Vec<u8>>> {
        let conn = self.pool.get().unwrap();

        let mut stmt = conn.prepare("SELECT binary FROM modules WHERE name = ?1")?;
        let mut rows = stmt.query(rusqlite::params![name])?;

        if let Some(row) = rows.next()? {
            let binary: Vec<u8> = row.get(0)?;
            Ok(Some(binary))
        } else {
            Ok(None)
        }
    }

    pub fn delet(&self, name: &str) -> Result<()> {
        self.pool.get().unwrap().execute("DELETE FROM modules WHERE name = ?1", params![name])?;
        Ok(())
    }
}
