// Internal Dependencies ------------------------------------------------------
use entity::State;
use renderer::Renderer;


// Bassic Level Traits --------------------------------------------------------
pub trait Base<S: State> {

    fn limit_state(&self, state: &mut S);

    fn interpolate_entity_state(&self,_: &mut Renderer,_: &S, _: &S) -> S;

    fn encoded_size() -> usize where Self: Sized;

    fn from_serialized(data: &[u8]) -> Self where Self: Sized;

    fn serialize(&self) -> Vec<u8>;

}

pub trait Drawable<S: State> {
    fn draw(&mut self, _: &mut Renderer, _: &Base<S>) {}
}

