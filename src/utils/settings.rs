use std::env;
use dotenvy::dotenv;


#[derive(Clone)]
pub struct AppSettings {
    pub db_connection: String,
}

impl AppSettings {
    pub fn new() -> Self {
        dotenv().ok();

        let db_connection = match env::var("db_connection") {
            Ok(v) => v.parse::<String>().unwrap_or(":memory:".to_owned()),
            Err(_) => ":memory:".to_owned(),
        };

        AppSettings {
            db_connection
        }
    }
}
