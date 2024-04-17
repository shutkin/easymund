use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EasymundEvent {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant: Option<Participant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambience: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participants: Option<Vec<Participant>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat: Option<Chat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambiences: Option<Vec<Ambience>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Participant {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub is_admin: Option<bool>,
    pub is_muted: Option<bool>,
    pub is_sharing: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ambience {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Chat {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub history: Option<Vec<ChatMessage>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    pub id: u64,
    pub from: String,
    pub text: String,
    pub time: String,
}

pub fn room(self_id: u64, name: String, participants: Vec<Participant>, ambiences: Vec<Ambience>, ambience: Option<String>, chat: Vec<ChatMessage>) -> EasymundEvent {
    let self_participant = participants.iter().find(|&participant| participant.id.unwrap_or_default() == self_id).cloned();
    EasymundEvent {
        event: String::from("room"),
        room_name: Some(name),
        participants: Some(participants),
        ambiences: Some(ambiences),
        ambience,
        participant: self_participant,
        chat: Some(Chat {
            message: None,
            history: Some(chat),
        }),
        error: None,
    }
}

pub fn participants(participants: Vec<Participant>) -> EasymundEvent {
    EasymundEvent {
        event: String::from("participants"),
        participants: Some(participants),
        room_name: None,
        ambiences: None,
        ambience: None,
        participant: None,
        chat: None,
        error: None,
    }
}

pub fn ambience(ambience: String) -> EasymundEvent {
    EasymundEvent {
        event: String::from("ambience"),
        room_name: None,
        participants: None,
        ambiences: None,
        ambience: Some(ambience),
        participant: None,
        chat: None,
        error: None,
    }
}

pub fn leave() -> EasymundEvent {
    EasymundEvent {
        event: String::from("leave"),
        room_name: None,
        participants: None,
        ambiences: None,
        ambience: None,
        participant: None,
        chat: None,
        error: None,
    }
}

pub fn chat_message(chat_message: ChatMessage) -> EasymundEvent {
    EasymundEvent {
        event: String::from("chat"),
        room_name: None,
        participants: None,
        ambiences: None,
        ambience: None,
        participant: None,
        chat: Some(Chat {
            message: None,
            history: Some(vec![chat_message]),
        }),
        error: None,
    }
}

pub fn error_event(error: String) -> EasymundEvent {
    EasymundEvent {
        event: String::from("error"),
        room_name: None,
        participants: None,
        ambiences: None,
        ambience: None,
        participant: None,
        chat: None,
        error: Some(error),
    }
}