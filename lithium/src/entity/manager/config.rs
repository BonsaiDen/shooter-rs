// External Dependencies ------------------------------------------------------
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};


// Internal Dependencies ------------------------------------------------------
#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct EntityManagerConfig {

    // Number of buffered ticks for client side rendering / server side rewinding
    pub buffered_ticks: u8,

    // Number of interpolation ticks for client side rendering
    pub interpolation_ticks: u8,

    // Ticks per second
    pub tick_rate: u8

}

impl EntityManagerConfig {

    pub fn encoded_size() -> usize {
        3
    }

    pub fn from_serialized(data: &[u8]) -> Self {
        decode::<Self>(data).unwrap()
    }

    pub fn serialize(&self) -> Vec<u8> {
        encode(&self, SizeLimit::Infinite).unwrap()
    }

}

