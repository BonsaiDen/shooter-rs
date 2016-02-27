// External Dependencies ------------------------------------------------------
use lithium;


// Game Events ----------------------------------------------------------------
#[derive(Debug, RustcEncodable, RustcDecodable)]
pub enum Event {
    PlayerJoined,
    PlayerLeft,
    Unknown
}

impl lithium::Event for Event {}

impl Default for Event {
    fn default() -> Event {
        Event::Unknown
    }
}

