use std::sync::mpsc::{channel, Sender};
use rusqlite::{Connection, Error, OptionalExtension, Result};

use super::{DbService, Query, AccessErr};


#[derive(Clone)]
pub struct PrevIpAccessor {
    sender: Sender<Query>,
}

pub enum PrevIpQueries {
    GetIps(String, Sender<Result<Vec<String>>>),
    AddIp(String, Sender<Result<()>>)
}

impl PrevIpAccessor {
    pub fn new(db_service: &DbService) -> Self {
        let sender = db_service.sender.clone();
        Self { sender }
    }

    pub fn get_prev_ips(&self, ip: String) -> Result<Vec<String>, AccessErr<Error>> {
        let (sender, receiver) = channel();
        let query = PrevIpQueries::GetIps(ip, sender);
        if let Err(_) = self.sender.send(Query::PrevIp(query)) { return Err(AccessErr::FailedToSend) };
        let res = if let Ok(v) = receiver.recv() { v } else { return Err(AccessErr::FailedToRecv) };
        match res {
            Ok(v) => Ok(v),
            Err(e) => Err(AccessErr::InnerError(e))
        }
    }

    pub fn add_user(&self, ip: String) -> Result<(), AccessErr<Error>> {
        let (sender, receiver) = channel();
        let query = PrevIpQueries::AddIp(ip, sender);
        if let Err(_) = self.sender.send(Query::PrevIp(query)) { return Err(AccessErr::FailedToSend) };
        if let Err(_) = receiver.recv() { return Err(AccessErr::FailedToRecv) };
        Ok(())
    }
}

#[allow(unused_must_use)]
pub fn run_prev_ip_query(query: PrevIpQueries, db: &Connection) {
    match query {
        PrevIpQueries::AddIp(ip, sender) => { sender.send(add_prev_ip(ip, db)); },
        PrevIpQueries::GetIps(ip, sender) => { sender.send(get_prev_ips(ip, db)); }
    };
}

fn get_prev_ips(ip: String, db: &Connection) -> Result<Vec<String>> {
    let mut stmt = db.prepare("SELECT address FROM prev_ip WHERE address LIKE (?1);")?;
    let res = stmt.query_map([format!("%{}", ip)], |r| r.get::<usize, String>(0))?;

    res.collect()
}

fn add_prev_ip(ip: String, db: &Connection) -> Result<()> {
    let prev_used_opt = db.query_row("SELCT used FROM prev_ip WHERE address = (?1)", 
            [&ip], 
            |r| r.get::<usize, usize>(0)
        )
        .optional()?;
    if let Some(prev_used) = prev_used_opt {
        return update_prev_ip(prev_used, db);
    }

    db.execute("INSERT INTO prev_ip (address, used) VALUES (?1, 0);", [&ip])?;
    db.execute("UPDATE prev_ip SET used = used + 1;", [])?;
    db.execute("DELETE FROM prev_ip WHERE used = 6;", [])?;

    Ok(())
}

fn update_prev_ip(prev_used: usize, db: &Connection) -> Result<()> {
    db.execute("UPDATE prev_ip
        SET used = CASE 
            WHEN used = (?1) THEN 0
            WHEN used < (?1) THEN used + 1
            ELSE used 
        END;", 
        [prev_used]
    )?;
    return Ok(());
}


