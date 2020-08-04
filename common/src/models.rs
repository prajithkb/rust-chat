use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub from : Participant,
    pub msg: String,
    pub display_rgb: Option<(u8, u8, u8)>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Participant {
    pub name : String,
    pub id: String
}