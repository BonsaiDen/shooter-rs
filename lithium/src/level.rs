// Internal Dependencies ------------------------------------------------------
use entity;
use renderer::Renderer;


// Level Trait ----------------------------------------------------------------
pub trait Level {

    fn limit_state(&self, state: &mut entity::State);

    fn interpolate_entity_state(
        &self,
        renderer: &mut Renderer,
        current: &entity::State, last: &entity::State

    ) -> entity::State;

    fn draw(&mut self, _: &mut Renderer) {}

    fn encoded_size() -> usize where Self: Sized;

    fn from_serialized(data: &[u8]) -> Self where Self: Sized;

    fn serialize(&self) -> Vec<u8>;

}

