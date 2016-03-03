// External Dependencies ------------------------------------------------------
use cobalt::ConnectionID;
use rustc_serialize::{Encodable, Decodable};


// Internal Dependencies ------------------------------------------------------
use renderer::Renderer;
use level::{Level, BaseLevel};
use entity::{EntityEvent, EntityInput};


// Basic Entity Traits --------------------------------------------------------
pub trait BaseEntity<S: EntityState, L: BaseLevel<S>> {

    fn type_id(&self) -> u8;

    fn apply_input(
        &mut self, level: &Level<S, L>, state: &mut S, input: &EntityInput, dt: f32
    );

    fn visible_to(&self, _: &ConnectionID) -> bool {
        true
    }

    fn serialize_state(&self, _: &mut S, _: &ConnectionID) {}

    fn event(&mut self, _: &EntityEvent, _: &S) {}

}

pub trait DrawableEntity<S: EntityState, L: BaseLevel<S>, R: Renderer> {

    fn draw(&mut self, _: &mut R, _: &Level<S, L>, _: S) {}

    fn event(&mut self, _: &EntityEvent, _: &S) {}

}

pub trait EntityState: Encodable + Decodable {

    fn encoded_size() -> usize where Self: Sized;

    fn from_serialized(data: &[u8]) -> Self where Self: Sized;

    fn serialize(&self) -> Vec<u8>;

    fn set_to(&mut self, state: &Self);

    fn clone(&self) -> Self;

    fn default() -> Self where Self: Sized;

    fn flags(&self) -> u8;

    fn set_flags(&mut self, u8);

}

