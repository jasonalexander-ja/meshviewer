use meshtastic::api::ConnectedStreamApi;
use meshtastic::protobufs::FromRadio;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;
use std::sync::mpsc::{channel, Receiver, Sender};


pub type Connection = (UnboundedReceiver<FromRadio>, ConnectedStreamApi);


pub struct ConnectionService {
    pub handle: JoinHandle<()>,
    pub to: Sender<()>,
    pub from: Receiver<()>
}

impl ConnectionService {
    pub fn start(decoded_listener: UnboundedReceiver<FromRadio>, stream_api: ConnectedStreamApi) -> Self {
        let (application_sender, service_rec) = channel();
        let (service_sender, application_rec) = channel();
        let handle = tokio::spawn(async move {
            
        });
        ConnectionService {
            handle,
            to: application_sender,
            from: application_rec
        }
    }
}
