use rusqlite::Connection;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::{thread, fmt};
use std::error::Error;

pub mod prev_ip;


#[derive(Debug)]
pub enum AccessErr<T: Error> {
    FailedToSend,
    FailedToRecv,
    InnerError(T)
}

impl<T: Error> fmt::Display for AccessErr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::FailedToSend => write!(f, "Failed to send request to DB service. "),
            Self::FailedToRecv => write!(f, "Failed to receive response from DB service. "),
            Self::InnerError(s) => write!(f, "Error thrown by query: \r\n{} ", s),
        }
    }
}

impl<T: Error + 'static> Error for AccessErr<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self {
            Self::InnerError(s) => Some(s),
            _ => None
        }
    }
}

pub enum Query {
    PrevIp(prev_ip::PrevIpQueries),
    End
}

pub struct DbService {
    sender: Sender<Query>,
    handle: thread::JoinHandle<()>,
}

impl DbService {
    fn new(sender: Sender<Query>, handle: thread::JoinHandle<()>) -> Self {
        DbService {
            sender,
            handle
        }
    }

    pub fn start_db_service(db: Connection) -> Self {
        let (sender, receiver) = channel();
        let handle = thread::spawn(move || {
            db_request_handler(receiver, db);
        });
        DbService::new(sender, handle)
    }

    pub fn end(self) {
        self.sender.send(Query::End).unwrap();
        self.handle.join().unwrap();
    }
}

fn db_request_handler(rec: Receiver<Query>, db: Connection) {
    loop {
        let msg = if let Ok(v) = rec.recv() { v } else { return };
        match msg {
            Query::PrevIp(q) => prev_ip::run_prev_ip_query(q, &db),
            Query::End => return
        }
    }
}

