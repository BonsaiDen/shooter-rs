// External Dependencies ------------------------------------------------------
use rand::XorShiftRng;
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
use entity;
use arena::Arena;
use renderer::Renderer;


// Basic Entity Traits --------------------------------------------------------
pub trait Base {

    fn typ(&self) -> u8;

    fn apply_inputs(
        &mut self,
        mut state: entity::State,
        &Vec<entity::Input>,
        arena: &Arena,
        dt: f32

    ) -> entity::State;

    fn visible_to(&self, _: &ConnectionID) -> bool {
        true
    }

    // Server / Client Specific Methods ---------------------------------------
    fn server_event_tick(&mut self, _: &Arena, _: &entity::State, _: u8, _: f32) {}
    fn client_event_tick(&mut self, _: &Arena, _: &entity::State, _: u8, _: f32) {}

    fn server_event_created(&mut self, _: &entity::State, _: u8) {}
    fn client_event_created(&mut self, _: &entity::State, _: u8) {}

    fn server_event_destroyed(&mut self, _: &entity::State, _: u8) {}
    fn client_event_destroyed(&mut self, _: &entity::State, _: u8) {}

    fn event_flags(&mut self, _: u8) {}

}

pub trait Drawable {

    fn draw(
        &mut self,
        _: &mut Renderer,
        _: &mut XorShiftRng,
        _: &Arena, _: entity::State, _: f32, _: f32
    ) {
    }

    fn event_flags(&mut self, _: u8) {}

    fn event_created(&mut self, _: &entity::State, _: u8) {}

    fn event_destroyed(&mut self, _: &entity::State, _: u8) {}

}

