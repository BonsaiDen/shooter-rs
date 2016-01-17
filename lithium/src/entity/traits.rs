// External Dependencies ------------------------------------------------------
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
use entity;
use level::Level;
use renderer::Renderer;


// Basic Entity Traits --------------------------------------------------------
pub trait Base {

    fn type_id(&self) -> u8;

    fn apply_input(
        &mut self,
        level: &Level, state: &mut entity::State, input: &entity::Input, dt: f32
    );

    fn visible_to(&self, _: &ConnectionID) -> bool {
        true
    }

    fn serialize_state(&self, _: &mut entity::State, _: &ConnectionID) {}

    fn event(&mut self, _: &entity::Event, _: &entity::State) {}

}

pub trait Drawable {

    fn draw(&mut self, _: &mut Renderer, _: &Level, _: entity::State) {}

    fn event(&mut self, _: &entity::Event, _: &entity::State) {}

}

