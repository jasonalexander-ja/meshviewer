use rusqlite::{Connection, Result, OptionalExtension};
use super::update_v1;


pub fn check_db(db: &Connection) -> Result<()> {
    let version = get_db_version(db)?;

    if version == 0 { 
        make_version_table(db)?;
        update_v1(db)?; 
    }

    Ok(())
}

pub fn update_db_version(ver: usize, db: &Connection) -> Result<()> {
    db.execute("DELETE FROM db_version;", ())?;
    db.execute(
        "INSERT INTO db_version (version) VALUES (?1)", 
        [ver]
    )?;

    Ok(())
}

pub fn make_version_table(db: &Connection) -> Result<()> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS db_version (
            version INTEGER NOT NULL
        );",
        ()
    )?;

    Ok(())
} 

pub fn get_db_version(db: &Connection) -> Result<usize> {
    let mut stmt = db.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'db_version';")?;
    let res = stmt.query_row([], |r| r.get::<usize, String>(0)).optional()?;

    if res == None { return Ok(0); }

    let mut stmt = db.prepare("SELECT version FROM db_version LIMIT 1")?;
    match stmt.query_row([], |r| r.get::<usize, usize>(1)).optional()? {
        Some(v) => Ok(v),
        None => Ok(0)
    }
}

