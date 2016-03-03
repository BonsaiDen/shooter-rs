// External Dependencies ------------------------------------------------------
use lithium::EntityState;
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};


// Entity State ---------------------------------------------------------------
#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct SharedState {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub mx: f32,
    pub my: f32,
    pub flags: u8
}

impl EntityState for SharedState {

    fn encoded_size() -> usize {
        21
    }

    fn from_serialized(data: &[u8]) -> SharedState {
        decode::<SharedState>(data).unwrap()
    }

    fn serialize(&self) -> Vec<u8> {
        encode(&self, SizeLimit::Infinite).unwrap()
    }

    fn set_to(&mut self, state: &Self) {
        self.x = state.x;
        self.y = state.y;
        self.r = state.r;
        self.mx = state.mx;
        self.my = state.my;
        self.flags = state.flags;
    }

    fn clone(&self) -> Self {
        SharedState {
            x: self.x,
            y: self.y,
            r: self.r,
            mx: self.mx,
            my: self.my,
            flags: self.flags
        }
    }

    fn default() -> Self where Self: Sized {
        SharedState {
            x: 0.0,
            y: 0.0,
            r: 0.0,
            mx: 0.0,
            my: 0.0,
            flags: 0
        }
    }

    fn flags(&self) -> u8 {
        self.flags
    }

    fn set_flags(&mut self, flags: u8) {
        self.flags = flags;
    }

}

