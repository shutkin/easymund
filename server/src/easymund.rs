use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Debug;
use std::fs::File;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::{debug, error, info};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use tokio::{task, time};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use wav::{BitDepth, Header};

use easymund_audio_codec::codec::{Codec, EasymundAudio};

use crate::ambience::Ambience;
use crate::dto;
use crate::event_handler::EventHandler;
use crate::httpserver::PostHandler;
use crate::wsserver::WSClientEvent;

const SAMPLE_RATE: usize = 44100;
const TALKING_LEVEL: f32 = 0.025;

pub struct Easymund {
    packet_size: usize,
    context: Context,
}

pub struct Client {
    pub room: String,
    stream: Vec<f32>,
    stream_send_position: usize,
    is_talking: bool,
    silence_counter: u32,
    codec: Codec,
    pub participant: Option<Participant>,
}

impl Client {
    fn new(room_id: &str, easymun_audio: &EasymundAudio, packet_size: usize) -> Client {
        Client {
            room: String::from(room_id),
            stream: vec![0.0; packet_size / 2],
            stream_send_position: 0,
            is_talking: false,
            silence_counter: 0,
            codec: easymun_audio.create_codec(packet_size).unwrap(),
            participant: None,
        }
    }
}

#[derive(Debug)]
pub struct Participant {
    pub name: String,
    pub is_admin: bool,
    pub is_muted: bool,
    pub is_sharing: bool,
}

#[derive(Clone)]
pub struct ChatMessage {
    pub id: u64,
    pub from: String,
    pub text: String,
    pub time: DateTime<Utc>,
}

pub struct Room {
    pub id: String,
    pub name: String,
    pub clients: HashSet<u64>,
    pub chat: Vec<ChatMessage>,
    pub ambience_id: String,
    pub ambience_position: usize,
}

impl Room {
    fn new(id: String, name: String, ambience_id: &str) -> Room {
        Room {
            id,
            name,
            clients: HashSet::new(),
            chat: Vec::new(),
            ambience_id: String::from(ambience_id),
            ambience_position: 0,
        }
    }
}

#[derive(Clone)]
pub struct Context {
    pub clients: Arc<Mutex<HashMap<u64, Client>>>,
    pub rooms: Arc<Mutex<HashMap<String, Room>>>,
    pub ambiences: Arc<Vec<Ambience>>,
}

struct EasymundPostHandler {
    pub context: Context,
}

#[derive(Deserialize)]
struct RoomCreatePostReq {
    name: String,
}
#[derive(Serialize)]
struct RoomCreatePostResp {
    room_id: String,
}

#[async_trait]
impl PostHandler for EasymundPostHandler {
    async fn handle(&self, path: &str, req_body: &[u8]) -> Option<Vec<u8>> {
        debug!("Handle POST req {}", path);
        match path {
            "/create" => {
                match self.handle_create(req_body).await {
                    Ok(resp) => Some(resp.into_bytes()),
                    Err(e) => {
                        error!("Failed to handle /create req: {:?}", e);
                        None
                    }
                }
            }
            _ => None
        }
    }
}

impl EasymundPostHandler {
    async fn handle_create(&self, req_body: &[u8]) -> Result<String, Box<dyn Error>> {
        let req: RoomCreatePostReq = serde_json::from_slice(req_body)?;
        let room_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 12);
        let room = Room::new(room_id.clone(), req.name, &self.context.ambiences[0].id);
        info!("Create room {} with id {}", &room.name, &room_id);
        self.context.rooms.lock().await.insert(room_id.clone(), room);
        let resp = serde_json::to_string(&RoomCreatePostResp { room_id })?;
        Ok(resp)
    }
}

impl Easymund {
    pub fn create() -> Self {
        let ambiences = Ambience::read_dir("sounds").unwrap_or_else(|e| {
            error!("Failed to read ambiences: {:?}", e);
            Vec::new()
        });
        Self {
            packet_size: easymund_audio_codec::default_packet_size(),
            context: Context {
                clients: Arc::new(Mutex::new(HashMap::new())),
                rooms: Arc::new(Mutex::new(HashMap::new())),
                ambiences: Arc::new(ambiences)
            }
        }
    }

    pub fn get_post_handler(&self) -> Box<dyn PostHandler> {
        Box::new(EasymundPostHandler {
            context: self.context.clone(),
        })
    }

    pub async fn start(&self, mut events_channel: Receiver<WSClientEvent>, command_channel: Sender<WSClientEvent>) -> Result<(), Box<dyn Error>> {
        let context_clone = self.context.clone();
        let easymund_audio = EasymundAudio::new(SAMPLE_RATE, 1, 16);
        let tick_time = 1_000_000_u64 * self.packet_size as u64 / SAMPLE_RATE as u64;
        let packet_size = self.packet_size;
        let sender = command_channel.clone();
        task::spawn(async move {
            let mut interval = time::interval(Duration::from_micros(tick_time));
            loop {
                interval.tick().await;
                Easymund::handle_tick(context_clone.clone(), &sender, packet_size).await;
            }
        });

        let context_clone = self.context.clone();
        let sender = command_channel.clone();
        while let Some(event) = events_channel.recv().await {
            if !event.is_connected {
                Easymund::handle_client_disconnect(event.client_id, &context_clone, &sender).await;
            } else if !context_clone.clients.lock().await.contains_key(&event.client_id) {
                Easymund::handle_client_connected(event.client_id, event.text_message.unwrap_or_default(),
                                                  &context_clone, &easymund_audio, packet_size, &sender).await;
            } else if let Some(text) = event.text_message {
                EventHandler::handle_client_event(event.client_id, text, &context_clone, &sender).await;
            } else if let Some(data) = event.binary_message {
                Easymund::handle_client_stream(event.client_id, data.as_slice(), &context_clone, &sender).await;
            }
        }
        Ok(())
    }

    async fn handle_client_connected(client_id: u64, path: String, context: &Context,
                                     easymund_audio: &EasymundAudio, packet_size: usize, sender: &Sender<WSClientEvent>) {
        let room_id = path.strip_prefix('/').map(String::from).unwrap_or(path);
        info!("Client {} connect to room {:?}", client_id, &room_id);
        let room_exists = context.rooms.lock().await.contains_key(&room_id);
        if room_exists {
            context.clients.lock().await.insert(client_id, Client::new(&room_id, easymund_audio, packet_size));
            context.rooms.lock().await.get_mut(room_id.as_str()).unwrap().clients.insert(client_id);
        } else {
            let event = dto::error_event(format!("Конференция {} не существует", &room_id));
            let json = serde_json::to_string(&event).unwrap();
            if let Err(e) = sender.send(WSClientEvent {
                client_id,
                is_connected: true,
                text_message: Some(json),
                binary_message: None,
            }).await {
                error!("Failed to send error event to client {}: {:?}", client_id, e);
            }
        }
    }

    async fn handle_client_disconnect(client_id: u64, context: &Context, sender: &Sender<WSClientEvent>) {
        info!("Client {} disconnected", client_id);
        let mut room_id = None;
        let mut new_admin = None;
        if let Some(client) = context.clients.lock().await.remove(&client_id) {
            if let Some(room) = context.rooms.lock().await.get_mut(client.room.as_str()) {
                room.clients.remove(&client_id);
                room_id = Some(client.room.clone());
                if let Some(participant) = &client.participant {
                    if participant.is_admin {
                        new_admin = room.clients.iter().copied().min();
                    }
                }
            }

            let wav_data = client.stream.iter().map(|f| (f * 32768.0) as i16).collect::<Vec<i16>>();
            let wav_header = Header::new(1, 1, SAMPLE_RATE as u32, 16);
            let wav_filename = format!("client_{}.wav", client_id);
            if let Ok(mut file) = File::create(&wav_filename) {
                if let Err(e) = wav::write(wav_header, &BitDepth::from(wav_data), &mut file) {
                    error!("Failed to save client stream: {:?}", e);
                } else {
                    info!("Client {} stream {} samples written to {}", client_id, client.stream.len(), &wav_filename);
                }
            }
        }
        
        if let Some(new_admin) = new_admin {
            if let Some(client) = context.clients.lock().await.get_mut(&new_admin) {
                if let Some(participant) = &mut client.participant {
                    participant.is_admin = true;
                } else {
                    error!("Unregistered client {} can't be new admin in room {:?}", new_admin, &room_id);
                }
            }
        }

        if let Some(room_id) = room_id {
            EventHandler::handle_room_update(client_id, &room_id, context, sender).await;
        }
    }

    async fn handle_client_stream(client_id: u64, data: &[u8], context: &Context, sender: &Sender<WSClientEvent>) {
        let first_byte = data[0];
        let data = &data[1..data.len()];
        match first_byte {
            0 => Easymund::handle_client_audio(client_id, data, context).await,
            1 => Easymund::handle_client_video(client_id, data, context, sender).await,
            _ => error!("Unknown stream type {}", first_byte)
        }
    }

    async fn handle_client_audio(client_id: u64, data: &[u8], context: &Context) {
        if let Some(client) = context.clients.lock().await.get_mut(&client_id) {
            match client.codec.decode(data) {
                Ok(decoded_res) => {
                    let decoded: Vec<Vec<f32>> = decoded_res;
                    client.stream.extend_from_slice(decoded[0].as_slice());
                }
                Err(e) => {error!("Failed to decode: {:?}", e);}
            }
        }
    }
    
    async fn handle_client_video(client_id: u64, data: &[u8], context: &Context, sender: &Sender<WSClientEvent>) {
        let mut room_id = None;
        if let Some(client) = context.clients.lock().await.get(&client_id) {
            room_id = Some(client.room.clone());
        }
        let mut send_futures = Vec::new();
        if let Some(room_id) = room_id {
            if let Some(room) = context.rooms.lock().await.get_mut(&room_id) {
                for other_client_id in room.clients.iter().copied() {
                    if other_client_id != client_id {
                        let mut frame = Vec::with_capacity(data.len() + 1);
                        frame.push(1);
                        frame.extend_from_slice(data);
                        let event = WSClientEvent {client_id: other_client_id, is_connected: true, text_message: None, binary_message: Some(frame)};
                        send_futures.push(sender.send(event));
                    }
                }
            }
        }
        for future in send_futures {
            if let Err(e) = future.await {
                error!("Failed to send video frame: {:?}", e);
            }
        }
    }

    async fn handle_tick(context: Context, sender: &Sender<WSClientEvent>, packet_size: usize) {
        let mut send_futures = Vec::new();
        for room in context.rooms.lock().await.values_mut() {
            let ambience_chunk = context.ambiences.iter().find(|a| a.id == room.ambience_id)
                .map(|ambience| Easymund::room_ambience_chunk(room, packet_size, &ambience.data));

            let mut talking_clients = Vec::new();
            let mut talking_clients_changes = false;
            let mut clients_chunks = HashMap::new();
            for client_id in &room.clients {
                if let Some(client) = context.clients.lock().await.get_mut(client_id) {
                    let client_chunk_length = client.stream.len() - client.stream_send_position;
                    let client_chunk_length = if client_chunk_length > packet_size {packet_size} else {client_chunk_length};
                    let mut client_chunk = Vec::with_capacity(client_chunk_length);
                    client_chunk.extend_from_slice(&client.stream[client.stream_send_position .. (client.stream_send_position + client_chunk_length)]);
                    if Easymund::check_talking_status(client, &client_chunk) {
                        talking_clients_changes = true;
                    }
                    if client.is_talking {
                        talking_clients.push(*client_id);
                    }
                    client.stream_send_position += client_chunk_length;
                    clients_chunks.insert(*client_id, client_chunk);
                }
            }
            let talking_clients_event_json = if talking_clients_changes {
                match serde_json::to_string(&dto::talking(talking_clients)) {
                    Ok(json) => Some(json),
                    Err(e) => {
                        error!("Failed to serialize talking event: {:?}", e);
                        None
                    }
                }
            } else { None };

            for client_id in &room.clients {
                let mut channels = Vec::new();
                if let Some(chunk) = &ambience_chunk {
                    if !chunk.is_empty() {
                        channels.push(chunk.as_slice());
                    }
                }
                for (other_client_id, other_client_chunk) in &clients_chunks {
                    if *client_id != *other_client_id {
                        channels.push(other_client_chunk);
                    }
                }
                let chunk = Easymund::mix(&channels);
                if !chunk.is_empty() {
                    let mut encoded: Option<Vec<u8>> = None;
                    if let Some(client) = context.clients.lock().await.get_mut(client_id) {
                        match client.codec.encode(vec![chunk.as_slice()].as_slice()) {
                            Ok(encoded_res) => { encoded = Some(encoded_res); }
                            Err(e) => { error!("Failed to encode: {:?}", e); }
                        }
                    }

                    if let Some(bytes) = encoded {
                        let mut frame = Vec::with_capacity(bytes.len() + 1);
                        frame.push(0);
                        frame.extend_from_slice(&bytes);
                        let event = WSClientEvent { client_id: *client_id, is_connected: true, text_message: None, binary_message: Some(frame) };
                        send_futures.push(sender.send(event));
                    }
                }
                if let Some(json) = talking_clients_event_json.clone() {
                    let event = WSClientEvent { client_id: *client_id, is_connected: true, text_message: Some(json), binary_message: None };
                    send_futures.push(sender.send(event));
                }
            }
        }
        for future in send_futures {
            if let Err(e) = future.await {
                error!("Failed to send audio frame: {:?}", e);
            }
        }
    }
    
    fn check_talking_status(client: &mut Client, chunk: &[f32]) -> bool {
        let average_level = chunk.iter()
            .map(|v| v.abs())
            .reduce(|acc, v| acc + v).unwrap_or_default() / (chunk.len() as f32 + 1.0);
        if average_level > TALKING_LEVEL {
            if !client.is_talking {
                client.is_talking = true;
                client.silence_counter = 0;
                true
            } else {
                false
            }
        } else if client.is_talking {
            client.silence_counter += 1;
            if client.silence_counter >= 5 {
                client.is_talking = false;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn room_ambience_chunk(room: &mut Room, samples_count: usize, background: &Vec<f32>) -> Vec<f32> {
        if background.len() < samples_count {
            return Vec::new();
        }
        let mut chunk = Vec::with_capacity(samples_count);
        let mut pos = room.ambience_position;
        for _ in 0..samples_count {
            pos = if pos >= background.len() - 1 {0} else {pos + 1};
            chunk.push(background[pos]);
        }
        room.ambience_position = pos;
        chunk
    }

    fn mix(channels: &[&[f32]]) -> Vec<f32> {
        if channels.is_empty() {
            return Vec::new()
        }
        let length = channels[0].len();
        let mut result = Vec::with_capacity(length);
        for i in 0..length {
            let mut v = 0.0;
            for channel in channels {
                if channel.len() > i {
                    v += channel[i];
                }
            }
            result.push(v);
        }
        result
    }
}
