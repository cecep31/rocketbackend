use rusqlite::{Connection, Result};

pub fn connect() -> Result<Connection> {
    let conn = Connection::open("database.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS posts (
                  id INTEGER PRIMARY KEY,
                  title TEXT NOT NULL,
                  body TEXT NOT NULL,
                  published_at TEXT NOT NULL
                  )",
        [],
    )?;
    Ok(conn)
}
