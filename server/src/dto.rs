use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EasymundEvent {
    pub event: String,
    pub name: Option<String>,
    pub ambience: Option<String>,
    pub participants: Option<Vec<Participant>>,
    pub ambiences: Option<Vec<Ambience>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Participant {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ambience {
    pub id: String,
    pub name: String,
}
