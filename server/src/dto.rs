use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EasymundEvent {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant: Option<Participant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambience: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participants: Option<Vec<Participant>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambiences: Option<Vec<Ambience>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Participant {
    pub name: Option<String>,
    pub is_muted: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ambience {
    pub id: String,
    pub name: String,
}

pub fn room(participants: Vec<Participant>, ambiences: Vec<Ambience>, ambience: Option<String>) -> EasymundEvent {
    EasymundEvent {
        event: String::from("room"),
        participants: Some(participants),
        ambiences: Some(ambiences),
        ambience,
        participant: None,
    }
}

pub fn participants(participants: Vec<Participant>) -> EasymundEvent {
    EasymundEvent {
        event: String::from("participants"),
        participants: Some(participants),
        ambiences: None,
        ambience: None,
        participant: None,
    }
}

pub fn ambience(ambience: String) -> EasymundEvent {
    EasymundEvent {
        event: String::from("ambience"),
        participants: None,
        ambiences: None,
        ambience: Some(ambience),
        participant: None
    }
}

pub fn leave() -> EasymundEvent {
    EasymundEvent {
        event: String::from("leave"),
        participants: None,
        ambiences: None,
        ambience: None,
        participant: None
    }
}