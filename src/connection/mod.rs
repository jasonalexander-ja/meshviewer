use inquire::{Text, Select, InquireError};
use meshtastic::api::{ConnectedStreamApi, StreamApi, StreamHandle};
use meshtastic::protobufs::FromRadio;
use meshtastic::errors::Error as MeshError;
use meshtastic::utils;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::UnboundedReceiver;
use serialport;
use std::error::Error;
use std::fmt;

use crate::cli::Params;
use crate::cli::autocompletes::prev_ip;
use crate::access::DbService;


pub type Connection = (UnboundedReceiver<FromRadio>, ConnectedStreamApi);

#[derive(Debug)]
pub enum ConnectError {
    MeshError(MeshError),
    InquireError(InquireError),
    SerialErr(serialport::Error)
}

impl fmt::Display for ConnectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MeshError(e) => write!(f, "Error opening Meshtastic device:\n\r{}", e),
            Self::InquireError(e) => write!(f, "Error polling the user:\n\r{}", e),
            Self::SerialErr(e) => 
                write!(f, "Error opening serial port to Meshtastic device:\n\r{}", e),
        }
    }
}

impl Error for ConnectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MeshError(e) => Some(e),
            Self::InquireError(e) => Some(e),
            Self::SerialErr(e) => Some(e),
        }
    }
}

pub async fn connect(config: &Params, db: &DbService) -> Result<Connection, ConnectError> {
    if let Some(v) = &config.port {
        return get_serial_connection(v).await
            .map_err(|e| ConnectError::MeshError(e))
    }

    if let Some(v) = &config.address {
        return get_tcp_connection(v).await
            .map_err(|e| ConnectError::MeshError(e))
    }
    
    poll_user_source(db).await
}

async fn poll_user_source(db: &DbService) -> Result<Connection, ConnectError> {
    let serial = Select::new("Connect via Serial or TCP", vec!["Serial", "TCP"])
        .prompt()
        .map_err(|e| ConnectError::InquireError(e))?;

    if serial == "Serial" {
        return poll_user_serial().await;
    }
    poll_user_ip(db).await
}

async fn poll_user_serial() -> Result<Connection, ConnectError> {
    let ports = utils::stream::available_serial_ports()
        .map_err(|e| ConnectError::SerialErr(e))?;
    let port = Select::new("Select a serial port", ports).prompt()
        .map_err(|e| ConnectError::InquireError(e))?;

    get_serial_connection(&port).await
        .map_err(|e| ConnectError::MeshError(e))
}

async fn poll_user_ip(db: &DbService) -> Result<Connection, ConnectError> {
    let prev_ip_acc = crate::access::prev_ip::PrevIpAccessor::new(db);
    let host = Text::new("What is the IP and port number?")
        .with_autocomplete(prev_ip::PrevIpAutocomplete::new(prev_ip_acc))
        .prompt()
        .map_err(|e| ConnectError::InquireError(e))?;

    get_tcp_connection(&host).await
        .map_err(|e| ConnectError::MeshError(e))
}

async fn get_serial_connection(port: &String) -> Result<Connection, MeshError> {
    let stream = utils::stream::build_serial_stream(port.clone(), None, None, None)?;
    make_connect_stream_api(stream).await
}

async fn get_tcp_connection(address: &String) -> Result<Connection, MeshError> {
    let stream = utils::stream::build_tcp_stream(address.clone()).await?;
    make_connect_stream_api(stream).await
}

async fn make_connect_stream_api<S>(stream_handle: StreamHandle<S>) -> Result<Connection, MeshError>
where
    S: AsyncReadExt + AsyncWriteExt + Send + 'static 
{
    let (decoded_listener, stream_api) = StreamApi::new().connect(stream_handle).await;

    let config_id = utils::generate_rand_id();
    let stream_api = stream_api.configure(config_id).await?;

    Ok((decoded_listener, stream_api))
}
