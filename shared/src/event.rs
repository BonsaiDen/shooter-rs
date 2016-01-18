// External Dependencies ------------------------------------------------------
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode, DecodingError, encoded_size};


// Game Events ----------------------------------------------------------------
#[derive(Debug, RustcEncodable, RustcDecodable)]
pub enum Event {
    PlayerJoined,
    PlayerLeft,
    Unknown
}

impl Event {

    pub fn encoded_size() -> usize {
        encoded_size::<Event>(&Event::Unknown) as usize
    }

    pub fn serialize(event: &Event) -> Vec<u8> {
        encode(event, SizeLimit::Infinite).unwrap()
    }

    pub fn from_serialized(data: &[u8]) -> Result<Event, DecodingError> {
        decode::<Event>(data)
    }

}

pub struct EventHandler {
    incoming: Option<Vec<Event>>,
    outgoing: Vec<Event>,
    event_size: usize
}

impl EventHandler {

    pub fn new() -> EventHandler {
        EventHandler {
            incoming: None,
            outgoing: Vec::new(),
            event_size: Event::encoded_size()
        }
    }

    pub fn send(&mut self, event: Event) {
        self.outgoing.push(event);
    }

    pub fn received(&mut self) -> Option<Vec<Event>> {
        self.incoming.take()
    }

    pub fn receive_events(&mut self, mut data: &[u8]) {

        let mut incoming = Vec::new();
        while let Ok(event) = Event::from_serialized(data) {
            incoming.push(event);
            data = &data[self.event_size..];
        }

        self.incoming = Some(incoming);

    }

    pub fn serialize_events(&mut self) -> Option<Vec<u8>> {

        if self.outgoing.len() > 0 {

            let mut data = [3].to_vec();
            for event in self.outgoing.iter() {
                data.extend(Event::serialize(event));
            }

            self.outgoing.clear();
            Some(data)

        } else {
            None
        }

    }

}

