use clap::Parser;
use rusqlite::Connection;
use colored::Colorize;

use utils::settings;
use access::DbService;

pub mod data;
pub mod access;
pub mod utils;
pub mod cli;
pub mod connection;


#[tokio::main]
async fn main() {

    let params = cli::Params::parse();
    let app_settings = settings::AppSettings::new();
    let db = open_seed_db(&app_settings);
    let (decoded_listener, stream_api) = connect_to_device(&params, &db).await;


    let _stream_api = stream_api.disconnect().await;
    println!("Hello, world!");
}

fn open_seed_db(app_settings: &settings::AppSettings) -> DbService {
    let db = match Connection::open(app_settings.db_connection.clone()) {
        Ok(v) => v,
        Err(e) => panic!("{}\r\n{:?}", "Failed to open database: ".yellow(), e)
    };

    match data::seed::check_db(&db) {
        Err(e) => panic!("{}\r\n{:?}", "Failed to initialise the database: ".yellow(), e),
        _ => ()
    };

    return DbService::start_db_service(db);
}

async fn connect_to_device(config: &cli::Params, db: &DbService) -> connection::Connection {
    match connection::connect(config, db).await {
        Ok(v) => v,
        Err(e) => panic!("{}\r\n{:?}", "Failed to open connection to Meshtastic device: ".yellow(), e)
    }
}
