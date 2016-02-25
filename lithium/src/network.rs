// Network Message Types ------------------------------------------------------
pub enum Message {
    ServerConfig = 0,
    ServerState = 1,
    ServerEvents = 2,
    ClientInput = 3,
    ClientEvents = 4,
    Unknown = 255
}

impl Message {
    pub fn from_u8(id: u8) -> Message {
        match id {
            0 => Message::ServerConfig,
            1 => Message::ServerState,
            2 => Message::ServerEvents,
            3 => Message::ClientInput,
            4 => Message::ClientEvents,
            _ => Message::Unknown
        }
    }
}

