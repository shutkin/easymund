use std::fs::File;
use std::io::Write;
use async_trait::async_trait;
use chrono::Utc;
use log::{debug, error, info};
use tokio::sync::mpsc::Sender;

use crate::dto;
use crate::easymund::{ChatMessage, Context, Participant};
use crate::wsserver::WSClientEvent;

struct ClientEvent {
    client_id: u64,
    event: dto::EasymundEvent,
}

#[async_trait]
trait Handler {
    async fn handle(&self, client_id: u64, room_id: &str, event: dto::EasymundEvent, context: &Context)
                    -> Vec<ClientEvent>;
}

struct JoinHandler {}

#[async_trait]
impl Handler for JoinHandler {
    async fn handle(&self, client_id: u64, room_id: &str, event: dto::EasymundEvent, context: &Context)
                    -> Vec<ClientEvent> {
        if let Some(client) = context.clients.lock().await.get_mut(&client_id) {
            let participant = Participant {
                name: event.participant.unwrap_or_default().name.unwrap_or(format!("{}", client_id)),
                is_muted: true,
            };
            info!("Client {}: {:?}", client_id, &participant);
            client.participant = Some(participant);
        }
        JoinHandler::join_events(client_id, room_id, context).await
    }
}

impl JoinHandler {
    async fn join_events(new_client_id: u64, room_id: &str, context: &Context) -> Vec<ClientEvent> {
        let mut participants = Vec::new();
        let mut other_clients_ids = Vec::new();
        let mut chat = Vec::new();
        let mut ambience = None;
        let mut room_name = None;
        if let Some(room) = context.rooms.lock().await.get(room_id) {
            ambience = Some(room.ambience_id.clone());
            room_name = Some(room.name.clone());
            for client_id in &room.clients {
                if let Some(client) = context.clients.lock().await.get(client_id) {
                    if let Some(participant) = &client.participant {
                        participants.push(participant_convert(*client_id, participant));
                        if *client_id != new_client_id {
                            other_clients_ids.push(*client_id);
                        }
                    }
                }
            }
            for message in &room.chat {
                chat.push(chat_msg_convert(message))
            }
        }
        let ambiences = context.ambiences.iter().map(|ambience| dto::Ambience {
            id: ambience.id.clone(),
            name: ambience.name.clone(),
        }).collect();

        let mut events = Vec::with_capacity(other_clients_ids.len() + 1);
        events.push(ClientEvent {
            client_id: new_client_id,
            event: dto::room(room_name.unwrap_or_default(), participants.clone(), ambiences, ambience, chat),
        });
        for client_id in other_clients_ids {
            events.push(ClientEvent {client_id, event: dto::participants(participants.clone())});
        }
        events
    }
}

struct LeaveHandler {}

#[async_trait]
impl Handler for LeaveHandler {
    async fn handle(&self, _: u64, room_id: &str, _: dto::EasymundEvent, context: &Context)
                    -> Vec<ClientEvent> {
        EventHandler::update_room_participants(room_id, context, None).await
    }
}

struct AmbienceHandler {}

#[async_trait]
impl Handler for AmbienceHandler {
    async fn handle(&self, _: u64, room_id: &str, event: dto::EasymundEvent, context: &Context)
                    -> Vec<ClientEvent> {
        let ambience = event.ambience.unwrap_or_default();
        let mut clients_ids = Vec::new();
        if let Some(room) = context.rooms.lock().await.get_mut(room_id) {
            room.ambience_id = ambience.clone();
            room.ambience_position = 0;
            for client_id in &room.clients {
                clients_ids.push(*client_id);
            }
        }
        let mut events = Vec::with_capacity(clients_ids.len());
        for client_id in clients_ids {
            events.push(ClientEvent {client_id, event: dto::ambience(ambience.clone())});
        }
        events
    }
}

struct ParticipantHandler {}

#[async_trait]
impl Handler for ParticipantHandler {
    async fn handle(&self, client_id: u64, room_id: &str, event: dto::EasymundEvent, context: &Context)
                    -> Vec<ClientEvent> {
        if let Some(client) = context.clients.lock().await.get_mut(&client_id) {
            if let Some(participant) = &mut client.participant {
                participant.is_muted = event.participant.unwrap_or_default().is_muted;
                info!("Participant {} is muted: {}", &participant.name, participant.is_muted);
            }
        }
        EventHandler::update_room_participants(room_id, context, None).await
    }
}

struct ChatHandler {}

#[async_trait]
impl Handler for ChatHandler {
    async fn handle(&self, client_id: u64, room_id: &str, event: dto::EasymundEvent, context: &Context)
                    -> Vec<ClientEvent> {
        let text = event.chat.unwrap_or_default().message.unwrap_or_default();
        let mut participant_name = String::new();
        if let Some(client) = context.clients.lock().await.get(&client_id) {
            if let Some(participant) = &client.participant {
                participant_name = participant.name.clone();
            }
        }

        let mut events = Vec::new();
        if let Some(room) = context.rooms.lock().await.get_mut(room_id) {
            let id = room.chat.len() as u64;
            let chat_message = ChatMessage {
                id,
                from: participant_name.clone(),
                text: text.clone(),
                time: Utc::now()
            };
            room.chat.push(chat_message.clone());
            for client_id in &room.clients {
                events.push(ClientEvent {
                    client_id: *client_id,
                    event: dto::chat_message(chat_msg_convert(&chat_message)),
                });
            }
        }
        events
    }
}

fn participant_convert(client_id: u64, participant: &Participant) -> dto::Participant {
    dto::Participant {
        id: Some(client_id),
        name: Some(participant.name.clone()),
        is_muted: participant.is_muted,
    }
}

fn chat_msg_convert(message: &ChatMessage) -> dto::ChatMessage {
    dto::ChatMessage {
        id: message.id,
        from: message.from.clone(),
        time: format!("{}", message.time.format("%H:%M:%S")),
        text: message.text.clone(),
    }
}

struct VideoStartHandler {}

#[async_trait]
impl Handler for VideoStartHandler {
    async fn handle(&self, client_id: u64, room_id: &str, _event: dto::EasymundEvent, _context: &Context) -> Vec<ClientEvent> {
        info!("Client {} started video stream in room {}", client_id, room_id);
        Vec::new()
    }
}

struct VideoStopHandler {}

#[async_trait]
impl Handler for VideoStopHandler {
    async fn handle(&self, client_id: u64, room_id: &str, _event: dto::EasymundEvent, context: &Context) -> Vec<ClientEvent> {
        info!("Client {} finished video stream in room {}", client_id, room_id);
        if let Some(room) = context.rooms.lock().await.get(room_id) {
            if !room.video.is_empty() {
                let filename = format!("{}.webm", room_id);
                if let Ok(mut file) = File::create(&filename) {
                    for chunk in &room.video {
                        if let Err(e) = file.write_all(chunk) {
                            error!("Failed to save room video: {:?}", e);
                        }
                    }
                }
            }
        }
        Vec::new()
    }
}

pub struct EventHandler {
}

impl EventHandler {
    async fn update_room_participants(room_id: &str, context: &Context, except_client: Option<u64>)
                                      -> Vec<ClientEvent> {
        let mut participants = Vec::new();
        let mut clients_ids = Vec::new();
        if let Some(room) = context.rooms.lock().await.get(room_id) {
            for client_id in &room.clients {
                if except_client.map_or(false, |id| *client_id == id) {
                    continue;
                }
                if let Some(client) = context.clients.lock().await.get(client_id) {
                    if let Some(participant) = &client.participant {
                        participants.push(participant_convert(*client_id, participant));
                        clients_ids.push(*client_id);
                    }
                }
            }
        }

        let mut events = Vec::with_capacity(clients_ids.len());
        for client_id in clients_ids {
            events.push(ClientEvent {client_id, event: dto::participants(participants.clone())});
        }
        events
    }


    fn get_handler(event: &str) -> Option<&dyn Handler> {
        match event {
            "join" => Some(&JoinHandler{}),
            "ambience" => Some(&AmbienceHandler{}),
            "participant" => Some(&ParticipantHandler{}),
            "chat" => Some(&ChatHandler{}),
            "video_start" => Some(&VideoStartHandler{}),
            "video_stop" => Some(&VideoStopHandler{}),
            _ => None
        }
    }

    pub async fn handle_client_event(client_id: u64, data: String, context: &Context, sender: &Sender<WSClientEvent>) {
        match serde_json::from_str::<dto::EasymundEvent>(data.as_str()) {
            Ok(event) => {
                debug!("Client {}: {:?}", client_id, &event);
                if let Some(handler) = EventHandler::get_handler(event.event.clone().as_str()) {
                    let mut room_id = None;
                    if let Some(client) = context.clients.lock().await.get(&client_id) {
                        room_id = Some(client.room.clone());
                    }
                    if let Some(room_id) = room_id {
                        EventHandler::handle_and_send(client_id, &room_id, event, handler, context, sender).await;
                    } else {
                        error!("Unknown client {}", client_id);
                    }
                } else {
                    error!("Unknown event '{}'", &event.event);
                }
            }
            Err(e) => {
                error!("Failed to deserialize {}: {:?}", &data, e);
            }
        }
    }

    async fn handle_and_send(client_id: u64, room_id: &str, event: dto::EasymundEvent, handler: &dyn Handler,
                             context: &Context, sender: &Sender<WSClientEvent>) {
        let events = handler.handle(client_id, room_id, event, context).await;
        for event in &events {
            let json = serde_json::to_string(&event.event).unwrap();
            if let Err(e) = sender.send(WSClientEvent {
                client_id: event.client_id,
                is_connected: true,
                text_message: Some(json),
                binary_message: None
            }).await {
                error!("Failed to send event to client {}: {:?}", event.client_id, e);
            }
        }
    }

    pub async fn handle_room_update(client_id: u64, room_id: &str, context: &Context, sender: &Sender<WSClientEvent>) {
        EventHandler::handle_and_send(client_id, room_id, dto::leave(), &LeaveHandler{}, context, sender).await;
    }
}