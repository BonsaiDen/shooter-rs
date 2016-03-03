// External Dependencies ------------------------------------------------------
use lithium::Event;


// Game Events ----------------------------------------------------------------
#[derive(Debug, RustcEncodable, RustcDecodable)]
pub enum SharedEvent {
    JoinGame,
    GameJoined,
    PlayerJoined,
    PlayerLeft,
    Unknown
}

impl Event for SharedEvent {}

impl Default for SharedEvent {
    fn default() -> SharedEvent {
        SharedEvent::Unknown
    }
}

