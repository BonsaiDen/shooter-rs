// External Dependencies ------------------------------------------------------
use bincode::SizeLimit;
use cobalt::ConnectionID;
use rustc_serialize::{Encodable, Decodable};
use bincode::rustc_serialize::{encode,decode_from};


// Abstract Event -------------------------------------------------------------
pub trait Event: Encodable + Decodable + Default {}


// Event Handler --------------------------------------------------------------
pub struct EventHandler<T: Event> {
    incoming: Option<Vec<(ConnectionID, T)>>,
    outgoing: Vec<(Option<ConnectionID>, T)>
}

impl<T: Event> EventHandler<T> {

    pub fn new() -> EventHandler<T> {
        EventHandler {
            incoming: None,
            outgoing: Vec::new()
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
        while let Ok(event) = decode_from::<&[u8], T>(
            &mut data, SizeLimit::Bounded(256)
        ) {
            incoming.push((owner_id, event));
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

