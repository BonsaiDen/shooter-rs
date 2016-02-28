// External Dependencies ------------------------------------------------------
use std::any::Any;


// Internal Dependencies ------------------------------------------------------
use entity;
use renderer::Renderer;


// Bassic Level Traits --------------------------------------------------------
pub trait Base<S: entity::State> {

    fn as_any(&mut self) -> &mut Any;

    fn limit_state(&self, state: &mut S);

    fn interpolate_entity_state(&self,_: &mut Renderer,_: &S, _: &S) -> S;

    fn encoded_size() -> usize where Self: Sized;

    fn from_serialized(data: &[u8]) -> Self where Self: Sized;

    fn serialize(&self) -> Vec<u8>;

}

pub trait Drawable<S: entity::State> {
    fn draw(&mut self, _: &mut Renderer, _: &Base<S>) {}
}

