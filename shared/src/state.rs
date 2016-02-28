// External Dependencies ------------------------------------------------------
use lithium::entity;
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};


// Entity State ---------------------------------------------------------------
#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct State {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub mx: f32,
    pub my: f32,
    pub flags: u8
}

impl entity::State for State {

    fn encoded_size() -> usize {
        21
    }

    fn from_serialized(data: &[u8]) -> State {
        decode::<State>(data).unwrap()
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
        State {
            x: self.x,
            y: self.y,
            r: self.r,
            mx: self.mx,
            my: self.my,
            flags: self.flags
        }
    }

    fn default() -> Self where Self: Sized {
        State {
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

