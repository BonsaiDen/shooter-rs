// External Dependencies ------------------------------------------------------
use lithium::Event;


// Internal Dependencies ------------------------------------------------------
use command::SharedCommand;

// Game Events ----------------------------------------------------------------
#[derive(Debug, RustcEncodable, RustcDecodable)]
pub enum SharedEvent {
    JoinGame,
    GameJoined,
    LeaveGame,
    PlayerJoined,
    PlayerLeft,
    Command(SharedCommand),
    Unknown
}

impl Event for SharedEvent {}

impl Default for SharedEvent {
    fn default() -> SharedEvent {
        SharedEvent::Unknown
    }
}

