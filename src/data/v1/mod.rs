use rusqlite::{Connection, Result};
use super::seed::update_db_version;

pub mod previous_ip;

pub use previous_ip::*;


pub fn update_v1(db: &Connection) -> Result<()> {
    prev_ip_create(db)?;

    db.execute(
        "UPDATE TABLE ", 
        ()
    )?;

    update_db_version(1, db)
}
