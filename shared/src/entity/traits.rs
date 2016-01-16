// External Dependencies ------------------------------------------------------
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
use entity;
use level::Level;
use renderer::Renderer;


// Basic Entity Traits --------------------------------------------------------
pub trait Base {

    fn typ(&self) -> u8;

    fn apply_input(
        &mut self,
        level: &Level, state: &mut entity::State, input: &entity::Input, dt: f32
    );

    fn visible_to(&self, _: &ConnectionID) -> bool {
        true
    }

    // Server / Client Specific Methods ---------------------------------------
    fn server_event_tick(&mut self, _: &Level, _: &entity::State, _: u8, _: f32) {}
    fn client_event_tick(&mut self, _: &Level, _: &entity::State, _: u8, _: f32) {}

    fn server_event_created(&mut self, _: &entity::State, _: u8) {}
    fn client_event_created(&mut self, _: &entity::State, _: u8) {}

    fn server_event_destroyed(&mut self, _: &entity::State, _: u8) {}
    fn client_event_destroyed(&mut self, _: &entity::State, _: u8) {}

    fn event_flags(&mut self, _: u8) {}

}

pub trait Drawable {

    fn draw(&mut self, _: &mut Renderer, _: &Level, _: entity::State) {}

    fn event_flags(&mut self, _: u8) {}

    fn event_created(&mut self, _: &entity::State, _: u8) {}

    fn event_destroyed(&mut self, _: &entity::State, _: u8) {}

}

