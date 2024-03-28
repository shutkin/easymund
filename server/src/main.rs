use std::error::Error;

use log::error;
use tokio::sync::mpsc;

use crate::easymund::Easymund;
use crate::httpserver::HTTPServer;
use crate::wsserver::WSServer;

mod wsserver;
mod easymund;
mod httpserver;
mod dto;
mod event_handler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    /*tokio::spawn(async {
        if let Err(e) = HTTPServer::start("[::]:80", "../client", false).await {
            error!("Failed to start HTTP server: {:?}", e);
        }
    });*/
    tokio::spawn(async {
        if let Err(e) = HTTPServer::start("[::]:443", "../client", true).await {
            error!("Failed to start HTTP server: {:?}", e);
        }
    });
    let (event_sender, event_receiver) = mpsc::channel(8);
    let (command_sender, command_receiver) = mpsc::channel(8);
    tokio::spawn(async move {
        if let Err(e) = WSServer::start(&String::from("[::]:5665"), event_sender, command_receiver).await {
            error!("Failed to start WebSocket server: {:?}", e);
        }
    });
    let easymund = Easymund::create();
    easymund.start(event_receiver, command_sender).await?;
    Ok(())
}
