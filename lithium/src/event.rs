// External Dependencies ------------------------------------------------------
use cobalt::ConnectionID;
use bincode::SizeLimit;
use bincode::rustc_serialize::{
    encode,
    decode,
    encoded_size
};

use rustc_serialize::{Encodable, Decodable};


// Abstract Event -------------------------------------------------------------
pub trait Event: Encodable + Decodable + Default {

}


// Event Handler --------------------------------------------------------------
pub struct Handler<T> where T: Event {
    incoming: Option<Vec<(ConnectionID, T)>>,
    outgoing: Vec<T>,
    event_size: usize
}

impl<T> Handler<T> where T: Event {

    pub fn new() -> Handler<T> {
        Handler {
            incoming: None,
            outgoing: Vec::new(),
            event_size: encoded_size::<T>(&T::default()) as usize
        }
    }

    pub fn send(&mut self, event: T) {
        self.outgoing.push(event);
    }

    pub fn received(&mut self) -> Option<Vec<(ConnectionID, T)>> {
        self.incoming.take()
    }

    pub fn receive_events(&mut self, owner_id: ConnectionID, mut data: &[u8]) {

        let mut incoming = Vec::new();
        while let Ok(event) =  decode::<T>(data) {
            incoming.push((owner_id, event));
            data = &data[self.event_size..];
        }

        self.incoming = Some(incoming);

    }

    pub fn serialize_events(&mut self) -> Option<Vec<u8>> {

        if self.outgoing.len() > 0 {

            let mut data = Vec::new();
            for event in self.outgoing.iter() {
                data.extend(encode(event, SizeLimit::Infinite).unwrap());
            }

            self.outgoing.clear();
            Some(data)

        } else {
            None
        }

    }

}

