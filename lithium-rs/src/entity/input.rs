// External Dependencies ------------------------------------------------------
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};


// Entity Input ---------------------------------------------------------------
#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct EntityInput {
    pub tick: u8,
    pub fields: u32
}

impl EntityInput {

    pub fn encoded_size() -> usize {
        5
    }

    pub fn from_serialized(data: &[u8]) -> EntityInput {
        match decode::<EntityInput>(data) {
            Ok(input) => input,
            Err(_) => EntityInput::default()
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        encode(&self, SizeLimit::Infinite).unwrap()
    }

}

impl Default for EntityInput {
    fn default() -> EntityInput {
        EntityInput {
            tick: 0,
            fields: 0
        }
    }
}

