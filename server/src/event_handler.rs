use async_trait::async_trait;
use log::{debug, error, info};
use tokio::sync::mpsc::Sender;

use crate::dto;
use crate::easymund::{Context, Participant};
use crate::wsserver::WSClientEvent;

struct ClientEvent {
    client_id: u64,
    event: dto::EasymundEvent,
}

#[async_trait]
trait Handler {
    async fn handle(&self, client_id: u64, room_name: &str, event: dto::EasymundEvent, context: &Context)
        -> Vec<ClientEvent>;
}

struct JoinHandler {}

#[async_trait]
impl Handler for JoinHandler {
    async fn handle(&self, client_id: u64, room_name: &str, event: dto::EasymundEvent, context: &Context)
        -> Vec<ClientEvent> {
        if let Some(client) = context.clients.lock().await.get_mut(&client_id) {
            let participant = Participant {
                name: event.name.unwrap_or_default()
            };
            info!("Client {}: {:?}", client_id, &participant);
            client.participant = Some(participant);
        }
        JoinHandler::join_events(client_id, room_name, context).await
    }
}

impl JoinHandler {
    async fn join_events(new_client_id: u64, room_name: &str, context: &Context) -> Vec<ClientEvent> {
        let mut participants = Vec::new();
        let mut other_clients_ids = Vec::new();
        let mut ambience = None;
        if let Some(room) = context.rooms.lock().await.get(room_name) {
            ambience = Some(room.ambience_id.clone());
            for client_id in &room.clients {
                if let Some(client) = context.clients.lock().await.get(client_id) {
                    if let Some(participant) = &client.participant {
                        participants.push(dto::Participant {name: participant.name.clone()});
                        if *client_id != new_client_id {
                            other_clients_ids.push(*client_id);
                        }
                    }
                }
            }
        }
        let ambiences = context.ambiences.iter().map(|ambience| dto::Ambience {
            id: ambience.id.clone(),
            name: ambience.name.clone(),
        }).collect();

        let mut events = Vec::with_capacity(other_clients_ids.len() + 1);
        events.push(ClientEvent {
            client_id: new_client_id,
            event: dto::room(participants.clone(), ambiences, ambience),
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
    async fn handle(&self, _: u64, room_name: &str, _: dto::EasymundEvent, context: &Context)
        -> Vec<ClientEvent> {
        let mut participants = Vec::new();
        let mut clients_ids = Vec::new();
        if let Some(room) = context.rooms.lock().await.get(room_name) {
            for client_id in &room.clients {
                if let Some(client) = context.clients.lock().await.get(client_id) {
                    if let Some(participant) = &client.participant {
                        participants.push(dto::Participant {name: participant.name.clone()});
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
}

struct AmbienceHandler {}

#[async_trait]
impl Handler for AmbienceHandler {
    async fn handle(&self, _: u64, room_name: &str, event: dto::EasymundEvent, context: &Context)
        -> Vec<ClientEvent> {
        let ambience = event.ambience.unwrap_or_default();
        let mut clients_ids = Vec::new();
        if let Some(room) = context.rooms.lock().await.get_mut(room_name) {
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

pub struct EventHandler {
}

impl EventHandler {
    fn get_handler(event: &str) -> Option<&dyn Handler> {
        match event {
            "join" => Some(&JoinHandler{}),
            "ambience" => Some(&AmbienceHandler{}),
            _ => None
        }
    }

    pub async fn handle_client_event(client_id: u64, data: String, context: &Context, sender: &Sender<WSClientEvent>) {
        match serde_json::from_str::<dto::EasymundEvent>(data.as_str()) {
            Ok(event) => {
                debug!("Client {}: {:?}", client_id, &event);
                if let Some(handler) = EventHandler::get_handler(event.event.clone().as_str()) {
                    let mut room_name = None;
                    if let Some(client) = context.clients.lock().await.get(&client_id) {
                        room_name = Some(client.room.clone());
                    }
                    if let Some(room_name) = room_name {
                        EventHandler::handle_and_send(client_id, &room_name, event, handler, context, sender).await;
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

    async fn handle_and_send(client_id: u64, room_name: &str, event: dto::EasymundEvent, handler: &dyn Handler,
                             context: &Context, sender: &Sender<WSClientEvent>) {
        let events = handler.handle(client_id, room_name, event, context).await;
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

    pub async fn handle_room_update(client_id: u64, room_name: &str, context: &Context, sender: &Sender<WSClientEvent>) {
        EventHandler::handle_and_send(client_id, room_name, dto::leave(), &LeaveHandler{}, context, sender).await;
    }
}