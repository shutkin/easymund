use std::error::Error;
use std::fs::File;

use log::{error, info};
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::easymund::Easymund;
use crate::httpserver::HTTPServer;
use crate::wsserver::WSServer;

mod wsserver;
mod easymund;
mod httpserver;
mod dto;
mod event_handler;
mod ambience;

#[derive(Deserialize, Debug)]
struct Config {
    http: HttpConfig,
}

#[derive(Deserialize, Debug)]
struct HttpConfig {
    is_secure: bool,
    content_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    let config_file = File::open("config/easymund.yaml").unwrap();
    let config: Config = serde_yaml::from_reader(config_file).unwrap();
    info!("{:?}", &config);

    let (event_sender, event_receiver) = mpsc::channel(8);
    let (command_sender, command_receiver) = mpsc::channel(8);
    tokio::spawn(async move {
        if let Err(e) = WSServer::start(&String::from("[::]:5665"), event_sender, command_receiver).await {
            error!("Failed to start WebSocket server: {:?}", e);
        }
    });
    let easymund = Easymund::create();
    let post_handler = easymund.get_post_handler();
    if config.http.is_secure {
        tokio::spawn(async {
            if let Err(e) = HTTPServer::start(config.http.content_path, true,
                                              post_handler).await {
                error!("Failed to start HTTP server: {:?}", e);
            }
        });
    } else {
        tokio::spawn(async {
            if let Err(e) = HTTPServer::start(config.http.content_path, false,
                                              post_handler).await {
                error!("Failed to start HTTP server: {:?}", e);
            }
        });
    }

    easymund.start(event_receiver, command_sender).await?;
    Ok(())
}
