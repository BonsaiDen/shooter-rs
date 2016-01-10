// External Dependencies ------------------------------------------------------
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};


// Entity State ---------------------------------------------------------------
#[derive(Debug, Copy, Clone, RustcEncodable, RustcDecodable)]
pub struct EntityState {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub mx: f32,
    pub my: f32,
    pub flags: u8
}

impl EntityState {

    pub fn encoded_size() -> usize {
        21
    }

    pub fn from_serialized(data: &[u8]) -> EntityState {
        let state: EntityState = decode(data).unwrap();
        state
    }

    pub fn serialize(&self) -> Vec<u8> {
        encode(&self, SizeLimit::Infinite).unwrap()
    }

}

impl Default for EntityState {
    fn default() -> EntityState {
        EntityState {
            x: 0.0,
            y: 0.0,
            r: 0.0,
            mx: 0.0,
            my: 0.0,
            flags: 0
        }
    }
}

