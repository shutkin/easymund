use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ParticipantEvent {
    pub event: String,
    pub name: Option<String>,
    pub participants: Option<Vec<String>>,
}
