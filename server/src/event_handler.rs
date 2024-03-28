use log::{debug, error, info};
use tokio::sync::mpsc::Sender;

use crate::dto;
use crate::easymund::{Context, Participant};
use crate::wsserver::WSClientEvent;

pub struct EventHandler {}

impl EventHandler {
    pub async fn handle_client_event(client_id: u64, data: String, context: &Context, sender: &Sender<WSClientEvent>) {
        match serde_json::from_str::<dto::ParticipantEvent>(data.as_str()) {
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
                    if let Some(room_name) = room_name {
                        EventHandler::send_room_participants(&room_name, context, sender).await;
                    }
                }
            }
            Err(e) => {
                error!("Failed to deserialize {}: {:?}", &data, e);
            }
        }
    }

    pub async fn handle_room_upate(room_name: &String, context: &Context, sender: &Sender<WSClientEvent>) {
        EventHandler::send_room_participants(room_name, context, sender).await;
    }

    async fn send_room_participants(room_name: &String, context: &Context, sender: &Sender<WSClientEvent>) {
        let mut participants = Vec::new();
        let mut clients_ids = Vec::new();
        if let Some(room) = context.rooms.lock().await.get(room_name) {
            for client_id in &room.clients {
                if let Some(client) = context.clients.lock().await.get_mut(client_id) {
                    if let Some(participant) = &client.participant {
                        participants.push(dto::Participant {name: participant.name.clone()});
                        clients_ids.push(*client_id);
                    }
                }
            }
        }
        let send_event = dto::ParticipantEvent {
            event: String::from("participants"),
            participants: Some(participants),
            name: None,
        };
        let json = serde_json::to_string(&send_event).unwrap();
        for client_id in clients_ids {
            debug!("Client {} send json {}", client_id, &json);
            if let Err(e) = sender.send(WSClientEvent { client_id, is_connected: true, text_message: Some(json.clone()), binary_message: None }).await {
                error!("Failed to send json event: {:?}", e);
            }
        }
    }
}