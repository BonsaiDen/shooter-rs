// External Dependencies ------------------------------------------------------
use cobalt::ConnectionID;
use rustc_serialize::{Encodable, Decodable};


// Internal Dependencies ------------------------------------------------------
use level::Level;
use entity::{Event, Input};
use renderer::Renderer;


// Basic Entity Traits --------------------------------------------------------
pub trait Base<S: State> {

    fn type_id(&self) -> u8;

    fn apply_input(
        &mut self, level: &Level<S>, state: &mut S, input: &Input, dt: f32
    );

    fn visible_to(&self, _: &ConnectionID) -> bool {
        true
    }

    fn serialize_state(&self, _: &mut S, _: &ConnectionID) {}

    fn event(&mut self, _: &Event, _: &S) {}

}

pub trait Drawable<S: State> {

    fn draw(&mut self, _: &mut Renderer, _: &Level<S>, _: S) {}

    fn event(&mut self, _: &Event, _: &S) {}

}

pub trait State: Encodable + Decodable {

    fn encoded_size() -> usize where Self: Sized;

    fn from_serialized(data: &[u8]) -> Self where Self: Sized;

    fn serialize(&self) -> Vec<u8>;

    fn set_to(&mut self, state: &Self);

    fn clone(&self) -> Self;

    fn default() -> Self where Self: Sized;

    fn flags(&self) -> u8;

    fn set_flags(&mut self, u8);

}

