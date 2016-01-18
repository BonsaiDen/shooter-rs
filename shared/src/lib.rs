extern crate lithium;
extern crate bincode;
extern crate rustc_serialize;

pub mod color;
pub mod entities;
pub mod event;
pub mod level;

pub enum NetworkMessage {
    ServerConfig = 0,
    ServerState = 1,
    ServerEvents = 2,
    ClientInput = 3,
    ClientEvents = 4,
    Unknown = 255
}

impl NetworkMessage {
    pub fn from_u8(id: u8) -> NetworkMessage {
        match id {
            0 => NetworkMessage::ServerConfig,
            1 => NetworkMessage::ServerState,
            2 => NetworkMessage::ServerEvents,
            3 => NetworkMessage::ClientInput,
            4 => NetworkMessage::ClientEvents,
            _ => NetworkMessage::Unknown
        }
    }
}

