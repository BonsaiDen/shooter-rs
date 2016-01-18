// Game Events ----------------------------------------------------------------
#[derive(Debug, RustcEncodable, RustcDecodable)]
pub enum Event {
    PlayerJoined,
    PlayerLeft,
    Unknown
}

impl Default for Event {
    fn default() -> Event {
        Event::Unknown
    }
}

