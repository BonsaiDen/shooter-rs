// Internal Dependencies ------------------------------------------------------
use entity;
use renderer::Renderer;


// Level Trait ----------------------------------------------------------------
pub trait Level<S: entity::State> {

    fn limit_state(&self, state: &mut S);

    fn interpolate_entity_state(
        &self,
        renderer: &mut Renderer,
        current: &S, last: &S

    ) -> S;

    fn draw(&mut self, _: &mut Renderer) {}

    fn encoded_size() -> usize where Self: Sized;

    fn from_serialized(data: &[u8]) -> Self where Self: Sized;

    fn serialize(&self) -> Vec<u8>;

}

