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
    outgoing: Vec<(Option<ConnectionID>, T)>,
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

    pub fn send(&mut self, receiver: Option<ConnectionID>, event: T) {
        self.outgoing.push((receiver, event));
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

    pub fn serialize_events(&mut self, receiver: Option<&ConnectionID>) -> Option<Vec<u8>> {

        let outgoing: Vec<u8> = self.outgoing.iter().filter(|event| {
            if let Some(r) = receiver {
                match event.0 {
                    Some(target) => {
                        target == *r
                    }
                    None => true
                }

            } else {
                true
            }

        }).fold(Vec::new(), |mut data, event| {
            data.extend(encode(&event.1, SizeLimit::Infinite).unwrap());
            data
        });

        if outgoing.len() > 0 {
            Some(outgoing)

        } else {
            None
        }

    }

    pub fn flush(&mut self) {
        self.outgoing.clear();
    }

}

