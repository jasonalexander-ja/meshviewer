use std::sync::mpsc::{channel, Receiver, Sender};
use rusqlite::{Connection, Error, OptionalExtension, Result};

use super::{DbService, Query, AccessErr};


#[derive(Clone)]
pub struct PrevIpAccessor {
    sender: Sender<Query>,
}

pub enum PrevIpQueries {
    GetIps(String, Sender<Result<Vec<String>>>),
    AddIp(String, Sender<Result<String>>)
}

impl PrevIpAccessor {
    pub fn new(db_service: &DbService) -> Self {
        let sender = db_service.sender.clone();
        Self { sender }
    }

    pub fn get_prev_ips(&self, ip: String) -> Result<Vec<String>, AccessErr<Error>> {
        let (sender, receiver) = channel();
        let query = PrevIpQueries::GetIps(ip, sender);
        self.issue_query(query, receiver)
    }

    pub fn add_address(&self, ip: String) -> Result<String, AccessErr<Error>> {
        let (sender, receiver) = channel();
        let query = PrevIpQueries::AddIp(ip, sender);
        self.issue_query(query, receiver)
    }

    fn issue_query<T>(&self, 
        query: PrevIpQueries, 
        receiver: Receiver<Result<T, Error>>
    ) -> Result<T, AccessErr<Error>> {
        self.sender.send(Query::PrevIp(query))
            .map_err(|_| AccessErr::FailedToSend)?;

        receiver.recv().map_err(|_| AccessErr::FailedToRecv)?
            .map_err(|e| AccessErr::InnerError(e))
    }
}

#[allow(unused_must_use)]
pub fn run_prev_ip_query(query: PrevIpQueries, db: &Connection) {
    match query {
        PrevIpQueries::AddIp(ip, sender) => sender.send(add_prev_ip(ip, db)).map_err(|_| ()),
        PrevIpQueries::GetIps(ip, sender) => sender.send(get_prev_ips(ip, db)).map_err(|_| ()),
    };
}

fn get_prev_ips(ip: String, db: &Connection) -> Result<Vec<String>> {
    let mut stmt = db.prepare("SELECT address FROM prev_ip WHERE address LIKE (?1);")?;
    let res = stmt.query_map([format!("%{}", ip)], |r| r.get::<usize, String>(0))?;

    res.collect()
}

fn add_prev_ip(ip: String, db: &Connection) -> Result<String> {
    let prev_used_opt = db.query_row("SELCT address, used FROM prev_ip WHERE address = (?1)", 
            [&ip], 
            |r| Ok((r.get::<usize, String>(0)?, r.get::<usize, usize>(1)?))
        )
        .optional()?;
    if let Some((ip, prev_used)) = prev_used_opt {
        update_prev_ip(prev_used, db)?;
        return Ok(ip)
    }

    db.execute("INSERT INTO prev_ip (address, used) VALUES (?1, 0);", [&ip])?;
    db.execute("UPDATE prev_ip SET used = used + 1;", [])?;
    db.execute("DELETE FROM prev_ip WHERE used = 6;", [])?;

    Ok(ip)
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


