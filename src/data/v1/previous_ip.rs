use rusqlite::{Connection, Result};


pub struct PrevIp {
    pub address: String,
    pub used: i64
}

pub fn prev_ip_create(db: &Connection) -> Result<()> {
    db.execute(
        "CREATE TABLE prev_ip (
            address  TEXT PRIMARY KEY,
            used     INTEGER NOT NULL
        );",
        ()
    )?;

    Ok(())
}
