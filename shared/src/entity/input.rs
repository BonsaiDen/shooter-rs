// External Dependencies ------------------------------------------------------
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};


// Entity Input ---------------------------------------------------------------
#[derive(Debug, Copy, Clone, RustcEncodable, RustcDecodable)]
pub struct EntityInput {
    pub tick: u8,
    pub left: bool,
    pub right: bool,
    pub thrust: bool,
    pub fire: bool
}

impl EntityInput {

    pub fn encoded_size() -> usize {
        5
    }

    pub fn from_serialized(data: &[u8]) -> EntityInput {
        let state: EntityInput = decode(data).unwrap();
        state
    }

    pub fn serialize(&self) -> Vec<u8> {
        encode(&self, SizeLimit::Infinite).unwrap()
    }

}

