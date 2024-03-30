use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EasymundEvent {
    pub event: String,
    pub name: Option<String>,
    pub ambience: Option<String>,
    pub participants: Option<Vec<Participant>>,
    pub ambiences: Option<Vec<Ambience>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Participant {
    pub name: String,
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
        name: None,
    }
}

pub fn participants(participants: Vec<Participant>) -> EasymundEvent {
    EasymundEvent {
        event: String::from("participants"),
        participants: Some(participants),
        ambiences: None,
        ambience: None,
        name: None,
    }
}

pub fn ambience(ambience: String) -> EasymundEvent {
    EasymundEvent {
        event: String::from("ambience"),
        participants: None,
        ambiences: None,
        ambience: Some(ambience),
        name: None
    }
}

pub fn leave() -> EasymundEvent {
    EasymundEvent {
        event: String::from("leave"),
        participants: None,
        ambiences: None,
        ambience: None,
        name: None
    }
}