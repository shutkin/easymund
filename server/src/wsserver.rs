use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::sync::Arc;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use log::{debug, error, info};
use sha1::{Digest, Sha1};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_rustls::TlsAcceptor;

use crate::httpserver::{HTTPGetReq, HTTPServer};

#[derive(Debug, Clone)]
pub struct WSClientEvent {
    pub client_id: u64,
    pub is_connected: bool,
    pub text_message: Option<String>,
    pub binary_message: Option<Vec<u8>>,
}

impl WSClientEvent {
    fn connected(client_id: u64, path: String) -> WSClientEvent {
        WSClientEvent {client_id, is_connected: true, text_message: Some(path), binary_message: None}
    }

    fn disconnected(client_id: u64) -> WSClientEvent {
        WSClientEvent {client_id, is_connected: false, text_message: None, binary_message: None}
    }

    fn text(client_id: u64, data: Vec<u8>) -> WSClientEvent {
        WSClientEvent {client_id, is_connected: true,
            text_message: Some(String::from_utf8(data).unwrap_or_default()),
            binary_message: None}
    }

    fn binary(client_id: u64, data: Vec<u8>) -> WSClientEvent {
        WSClientEvent {client_id, is_connected: true, text_message: None, binary_message: Some(data)}
    }
}

pub struct WSServer {}

enum WSOpcode {
    Text = 1, Binary = 2, Close = 8, Ping = 9, Pong = 10,
}

const WSCOMMAND_HANDSHAKE_OPCODE: i8 = -1;
const WSCOMMAND_CLOSE_OPCODE: i8 = -2;
struct WSCommand {
    opcode: i8,
    data: Vec<u8>,
}

impl WSCommand {
    fn close() -> WSCommand {
        WSCommand {opcode: WSCOMMAND_CLOSE_OPCODE, data: Vec::new()}
    }
}

#[derive(Clone)]
struct Context {
    addresses: Arc<Mutex<HashSet<String>>>,
    senders: Arc<Mutex<HashMap<u64, Sender<WSCommand>>>>,
}

impl WSServer {
    pub async fn start(addr: &String, events_channel: Sender<WSClientEvent>, command_channel: Receiver<WSClientEvent>) -> Result<(), Box<dyn Error>> {
        let tls_config = Arc::new(HTTPServer::create_tls_config()?);
        let tls_acceptor = TlsAcceptor::from(tls_config);

        let mut client_counter = 0_u64;
        let addresses = HashSet::new();
        let senders = HashMap::new();
        let context = Context {
            addresses: Arc::new(Mutex::new(addresses)),
            senders: Arc::new(Mutex::new(senders)),
        };
        let listener = TcpListener::bind(addr).await?;
        info!("WSServer started on {addr}");

        let context_clone = context.clone();
        tokio::spawn(async move {
            if let Err(e) = WSServer::send_commands(command_channel, context_clone).await {
                error!("Failed to send command: {:?}", e);
            }
        });

        loop {
            let (stream, addr) = listener.accept().await?;
            let addr = format!("{}", addr.ip());
            if context.addresses.lock().await.contains(&addr) {
                debug!("Ignore already connected client {:?}", &addr);
            } else {
                match tls_acceptor.clone().accept(stream).await {
                    Ok(stream) => {
                        let client_id = client_counter;
                        client_counter += 1;
                        info!("Accepted connection from {:?}, client_id={client_counter}", &addr);
                        //context.addresses.lock().await.insert(addr.clone());
                        let (sender, receiver) = mpsc::channel(8);
                        context.senders.lock().await.insert(client_id, sender.clone());

                        let (mut input, mut output) = tokio::io::split(stream);
                        let sender_clone = sender.clone();
                        let context_clone = context.clone();
                        let events_clone = events_channel.clone();
                        tokio::spawn(async move {
                            match WSServer::reader(&mut input, client_id, &sender_clone, &events_clone).await {
                                Ok((code, msg)) => {
                                    info!("Client {:?} exit with code: {:?}, message: {:?}", &addr, code, msg);
                                },
                                Err(e) => {
                                    error!("Client error: {:?}", e);
                                }
                            }
                            WSServer::handle_client_close(&sender_clone, &addr, client_id,
                                                          &events_clone, context_clone).await;
                        });
                        tokio::spawn(async move {
                            WSServer::writer(&mut output, receiver).await;
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept TLS: {:?}", e);
                    }
                }
            }
        }
    }

    async fn send_commands(mut receiver: Receiver<WSClientEvent>, context: Context) -> Result<(), Box<dyn Error>> {
        while let Some(event) = receiver.recv().await {
            if let Some(client_sender) = context.senders.lock().await.get(&event.client_id) {
                if let Some(text) = event.text_message {
                    client_sender.send(WSCommand {opcode: WSOpcode::Text as i8, data: text.into_bytes()}).await?;
                } else if let Some(data) = event.binary_message {
                    client_sender.send(WSCommand {opcode: WSOpcode::Binary as i8, data}).await?;
                } else {
                    error!("Send command without text or data {:?}", &event);
                }
            } else {
                error!("Send command to unknown client {}", event.client_id);
            }
        }
        Ok(())
    }

    async fn handle_client_close(client_sender: &Sender<WSCommand>, client_addr: &str, client_id: u64,
                                 events_sender: &Sender<WSClientEvent>, context: Context) {
        if let Err(e) = events_sender.send(WSClientEvent::disconnected(client_id)).await {
            error!("Failed to channel disconnected event {:?}", e);
        }
        context.addresses.lock().await.remove(client_addr);
        context.senders.lock().await.remove(&client_id);
        if let Err(e) = client_sender.send(WSCommand::close()).await {
            error!("Failed to channel close command: {:?}", e);
        }
    }

    async fn reader<T>(stream: &mut T, client_id: u64, client_sender: &Sender<WSCommand>,
                    events_sender: &Sender<WSClientEvent>) -> Result<(Option<u16>, Option<String>), Box<dyn Error>>
        where T: AsyncReadExt + Unpin {
        let handshake = HTTPServer::read_http_req(stream).await?;
        debug!("Handshake: {:?}", &handshake);
        let (path, handshake_response) = WSServer::parse_handshake(&handshake)?;
        debug!("Handshake response: {:?}", &handshake_response);
        let command = WSCommand {opcode: WSCOMMAND_HANDSHAKE_OPCODE, data: handshake_response.into_bytes()};
        client_sender.send(command).await?;
        if let Err(e) = events_sender.send(WSClientEvent::connected(client_id, path)).await {
            error!("Failed to channel connected event {:?}", e);
        }
        loop {
            let (opcode, data) = WSServer::read_message(stream).await?;
            if opcode == WSOpcode::Text as u8 {
                if let Err(e) = events_sender.send(WSClientEvent::text(client_id, data)).await {
                    error!("Failed to channel text event {:?}", e);
                }
            } else if opcode == WSOpcode::Binary as u8 {
                if let Err(e) = events_sender.send(WSClientEvent::binary(client_id, data)).await {
                    error!("Failed to channel binary event {:?}", e);
                }
            } else if opcode == WSOpcode::Close as u8 {
                let code = if data.len() > 1 { Some(data[0] as u16 + data[1] as u16) } else { None };
                let msg = if data.len() > 2 {
                    let (_, msg) = data.split_at(2);
                    Some(String::from_utf8(Vec::from(msg)).unwrap_or_default())
                } else { None };
                return Ok((code, msg));
            } else if opcode == WSOpcode::Ping as u8 {
                let command = WSCommand {opcode: WSOpcode::Pong as i8, data: Vec::new() };
                client_sender.send(command).await?;
            }
        }
    }

    async fn read_message<T>(stream: &mut T) -> Result<(u8, Vec<u8>), Box<dyn Error>>
        where T: AsyncReadExt + Unpin {
        let header0 = stream.read_u8().await?;
        let _fin = header0 & 0x80 > 0;
        let opcode = header0 & 0x0F;
        let header1 = stream.read_u8().await?;
        let mask = header1 & 0x80 > 0;
        if !mask {
            return Err(Box::new(WSError::new(String::from("Message without mask"))));
        }
        let length = header1 & 0x7F;
        let length = if length <= 125 { length as usize }
        else if length == 126 {
            stream.read_u16().await? as usize
        } else {
            stream.read_u64().await? as usize
        };
        let mut mask = vec![0_u8; 4];
        stream.read_exact(&mut mask).await?;
        let mut payload = vec![0_u8; length];
        stream.read_exact(&mut payload).await?;
        for i in 0..payload.len() {
            payload[i] ^= mask[i % 4];
        }
        Ok((opcode, payload))
    }

    fn parse_handshake(handshake: &HTTPGetReq) -> Result<(String, String), WSError> {
        let path = handshake.path.clone();
        let upgrade = handshake.headers.get(&String::from("Upgrade"));
        let ws_key = handshake.headers.get(&String::from("Sec-WebSocket-Key"));
        if path.is_none() || upgrade.is_none() || ws_key.is_none() {
            Err(WSError::new(format!("Invalid handshake {:?}", handshake)))
        } else {
            Ok((path.unwrap(), WSServer::generate_handshake_response(&ws_key.unwrap())))
        }
    }

    fn generate_handshake_response(client_key: &String) -> String {
        debug!("Client key: {client_key}");
        let str = format!("{client_key}258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
        let hash = Sha1::digest(str.into_bytes());
        let key = BASE64_STANDARD.encode(hash);
        format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {key}\r\n\r\n")
    }

    async fn writer<T>(stream: &mut T, mut client_receiver: Receiver<WSCommand>)
        where T: AsyncWriteExt + Unpin {
        while let Some(command) = client_receiver.recv().await {
            if command.opcode == WSCOMMAND_CLOSE_OPCODE {
                info!("Client write close");
                break;
            } else if command.opcode == WSCOMMAND_HANDSHAKE_OPCODE {
                if let Err(e) = stream.write_all(command.data.as_slice()).await {
                    error!("Failed to send handshake response: {:?}", e);
                }
            } else {
                let message_data = WSServer::generate_message(&command);
                if let Err(e) = stream.write_all(message_data.as_slice()).await {
                    error!("Failed to send message: {:?}", e);
                }
            }
        }
    }

    fn generate_message(command: &WSCommand) -> Vec<u8> {
        let mut result = Vec::new();
        result.push(0x80 | command.opcode as u8);
        if command.data.len() <= 125 {
            result.push(command.data.len() as u8);
        } else if command.data.len() < 65536 {
            result.push(126);
            result.push((command.data.len() >> 8) as u8);
            result.push(command.data.len() as u8);
        } else {
            let len_bytes = command.data.len().to_be_bytes();
            result.push(127);
            for i in 0..8 {
                let be_index = 7 - i;
                result.push(if be_index < len_bytes.len() { len_bytes[be_index] } else { 0 });
            }
        }
        if !command.data.is_empty() {
            result.extend_from_slice(command.data.as_slice());
        }
        result
    }
}

#[derive(Clone, Debug)]
struct WSError {
    description: String,
}
impl WSError {
    fn new(description: String) -> WSError {
        WSError {description}
    }
}
impl Error for WSError {}
impl fmt::Display for WSError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "WS Error: {:?}", &self.description)
    }
}