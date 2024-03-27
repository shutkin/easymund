use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Debug;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use log::{debug, error, info};
use tokio::{task, time};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use wav::{BitDepth, Header};

use easymund_audio_codec::codec::{Codec, EasymundAudio};

use crate::dto::ParticipantEvent;
use crate::wsserver::WSClientEvent;

const SAMPLE_RATE: usize = 44100;

pub struct Easymund {
    packet_size: usize,
}

struct Client {
    room: String,
    stream: Vec<f32>,
    stream_send_position: usize,
    codec: Codec,
    participant: Option<Participant>,
}

impl Client {
    fn new(room_name: &str, easymun_audio: &EasymundAudio, packet_size: usize) -> Client {
        Client {
            room: String::from(room_name),
            stream: Vec::new(),
            stream_send_position: 0,
            codec: easymun_audio.create_codec(packet_size).unwrap(),
            participant: None,
        }
    }
}

#[derive(Debug)]
struct Participant {
    name: String,
}

struct Room {
    clients: HashSet<u64>,
    background_position: usize,
}

impl Room {
    fn new() -> Room {
        Room {
            clients: HashSet::new(),
            background_position: 0,
        }
    }
}

#[derive(Clone)]
struct Context {
    clients: Arc<Mutex<HashMap<u64, Client>>>,
    rooms: Arc<Mutex<HashMap<String, Room>>>,
    last_tick_time: Arc<Mutex<SystemTime>>,
}

impl Easymund {
    pub fn create() -> Self {
        Self {
            packet_size: easymund_audio_codec::default_packet_size(),
        }
    }

    pub async fn start(&self, mut events_channel: Receiver<WSClientEvent>, command_channel: Sender<WSClientEvent>) -> Result<(), Box<dyn Error>> {
        let context = Context {
            clients: Arc::new(Mutex::new(HashMap::new())),
            rooms: Arc::new(Mutex::new(HashMap::new())),
            last_tick_time: Arc::new(Mutex::new(SystemTime::now())),
        };
        let background = Easymund::read_sound("sounds/forest-ambience.wav", 0.25)?;
        let background_arc = Arc::new(background);
        let background_clone = background_arc.clone();
        let context_clone = context.clone();
        let easymund_audio = EasymundAudio::new(SAMPLE_RATE, 1, 16);
        let tick_time = 1_000_000_u64 * self.packet_size as u64 / SAMPLE_RATE as u64;
        let packet_size = self.packet_size;
        let sender = command_channel.clone();
        task::spawn(async move {
            let mut interval = time::interval(Duration::from_micros(tick_time));
            loop {
                interval.tick().await;
                Easymund::handle_tick(background_clone.clone(), context_clone.clone(), &sender, packet_size).await;
            }
        });

        let context_clone = context.clone();
        let sender = command_channel.clone();
        while let Some(event) = events_channel.recv().await {
            if !event.is_connected {
                Easymund::handle_client_disconnect(event.client_id, &context_clone).await;
            } else if !context_clone.clients.lock().await.contains_key(&event.client_id) {
                Easymund::handle_client_connected(event.client_id, event.text_message.unwrap_or_default(), &context_clone, &easymund_audio, packet_size).await;
            } else if let Some(text) = event.text_message {
                Easymund::handle_client_event(event.client_id, text, &context_clone, &sender).await;
            } else if let Some(data) = event.binary_message {
                Easymund::handle_client_stream(event.client_id, data.as_slice(), &context_clone).await;
            }
        }
        Ok(())
    }

    fn read_sound(path: &str, factor: f32) -> Result<Vec<f32>, Box<dyn Error>> {
        let mut background_file = File::open(path)?;
        let (header, bits) = wav::read(&mut background_file)?;
        let data_i16 = bits.try_into_sixteen().unwrap_or_default();
        info!("Sound {} {:?}, length {}", path, &header, data_i16.len());
        Ok(data_i16.iter().map(|v| *v as f32 * factor / 32768.0).collect())
    }

    async fn handle_client_connected(client_id: u64, path: String, context: &Context, easymund_audio: &EasymundAudio, packet_size: usize) {
        let room_name = path.strip_prefix('/').map(String::from).unwrap_or(path);
        info!("Client {} connect to room {:?}", client_id, &room_name);
        context.clients.lock().await.insert(client_id, Client::new(&room_name, easymund_audio, packet_size));
        let mut lock = context.rooms.lock().await;
        if !lock.contains_key(&room_name) {
            lock.insert(room_name.clone(), Room::new());
        }
        lock.get_mut(room_name.as_str()).unwrap().clients.insert(client_id);
    }

    async fn handle_client_disconnect(client_id: u64, context: &Context) {
        info!("Client {} disconnected", client_id);
        if let Some(client) = context.clients.lock().await.remove(&client_id) {
            if let Some(room) = context.rooms.lock().await.get_mut(client.room.as_str()) {
                room.clients.remove(&client_id);
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
    }

    async fn handle_client_event(client_id: u64, data: String, context: &Context, sender: &Sender<WSClientEvent>) {
        match serde_json::from_str::<ParticipantEvent>(data.as_str()) {
            Ok(event) => {
                debug!("Client {}: {:?}", client_id, &event);
                if let Some(participant_name) = event.name {
                    let mut room_name = None;
                    if let Some(client) = context.clients.lock().await.get_mut(&client_id) {
                        let participant = Participant {name: participant_name};
                        info!("Client {}: {:?}", client_id, &participant);
                        client.participant = Some(participant);
                        room_name = Some(client.room.clone());
                    }
                    let mut participants = Vec::new();
                    if let Some(room) = context.rooms.lock().await.get(&room_name.unwrap()) {
                        for client_id in &room.clients {
                            if let Some(client) = context.clients.lock().await.get_mut(client_id) {
                                if let Some(participant) = &client.participant {
                                    participants.push(participant.name.clone());
                                }
                            }
                        }
                    }
                    let send_event = ParticipantEvent {
                        event: String::from("participants"),
                        participants: Some(participants),
                        name: None,
                    };
                    let json = serde_json::to_string(&send_event).unwrap();
                    debug!("Client {} send json {}", client_id, &json);
                    if let Err(e) = sender.send(WSClientEvent {client_id, is_connected: true, text_message: Some(json), binary_message: None}).await {
                        error!("Failed to send json event: {:?}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to deserialize {}: {:?}", &data, e);
            }
        }
    }

    async fn handle_client_stream(client_id: u64, data: &[u8], context: &Context) {
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

    async fn handle_tick(background: Arc<Vec<f32>>, context: Context, sender: &Sender<WSClientEvent>, packet_size: usize) {
        let now = SystemTime::now();
        let micros = now.duration_since(*context.last_tick_time.lock().await).unwrap_or_default().as_micros();
        let real_samples_count = (micros * SAMPLE_RATE as u128 / 1_000_000) as usize;
        *context.last_tick_time.lock().await = now;

        let mut send_futures = Vec::new();
        for room in context.rooms.lock().await.values_mut() {
            let background_chunk = Easymund::room_background_chunk(room, packet_size, &background);

            let mut clients_chunks = HashMap::new();
            for client_id in &room.clients {
                if let Some(client) = context.clients.lock().await.get_mut(client_id) {
                    let client_chunk_length = client.stream.len() - client.stream_send_position;
                    let client_chunk_length = if client_chunk_length > packet_size {packet_size} else {client_chunk_length};
                    let mut client_chunk = Vec::with_capacity(client_chunk_length);
                    client_chunk.extend_from_slice(&client.stream[client.stream_send_position .. (client.stream_send_position + client_chunk_length)]);
                    client.stream_send_position += client_chunk_length;
                    clients_chunks.insert(*client_id, client_chunk);
                }
            }

            for client_id in &room.clients {
                let mut channels = Vec::new();
                channels.push(background_chunk.as_slice());
                for (other_client_id, other_client_chunk) in &clients_chunks {
                    if *client_id != *other_client_id {
                        channels.push(other_client_chunk);
                    }
                }
                let chunk = Easymund::mix(&channels);

                let mut encoded: Option<Vec<u8>> = None;
                if let Some(client) = context.clients.lock().await.get_mut(client_id) {
                    match client.codec.encode(vec![chunk.as_slice()].as_slice()) {
                        Ok(encoded_res) => {encoded = Some(encoded_res);}
                        Err(e) => {error!("Failed to encode: {:?}", e);}
                    }
                }

                if let Some(bytes) = encoded {
                    let event = WSClientEvent {client_id: *client_id, is_connected: true, text_message: None, binary_message: Some(bytes.clone())};
                    send_futures.push(sender.send(event));
                }
            }
        }
        for future in send_futures {
            if let Err(e) = future.await {
                error!("Failed to send: {:?}", e);
            }
        }
    }

    fn room_background_chunk(room: &mut Room, samples_count: usize, background: &Arc<Vec<f32>>) -> Vec<f32> {
        let mut chunk = Vec::with_capacity(samples_count);
        let mut pos = room.background_position;
        for _ in 0..samples_count {
            chunk.push(background[pos]);
            pos = if pos == background.len() - 1 {0} else {pos + 1};
        }
        room.background_position = pos;
        chunk
    }

    fn mix(channels: &[&[f32]]) -> Vec<f32> {
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
