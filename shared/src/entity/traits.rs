// External Dependencies ------------------------------------------------------
use rand::XorShiftRng;
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
use entity;
use arena::Arena;
use renderer::Renderer;


// Basic Entity Traits --------------------------------------------------------
pub trait Base : Eventful {

    fn typ(&self) -> u8;

    fn apply_inputs(
        &mut self, mut state: entity::State, &Vec<entity::Input> , arena: &Arena, dt: f32

    ) -> entity::State;

    fn visible_to(&self, _: &ConnectionID) -> bool {
        true
    }

}

pub trait Drawable : Eventful {

    fn draw(
        &mut self,
        _: &mut Renderer,
        _: &mut XorShiftRng,
        _: &Arena, _: entity::State, _: f32, _: f32
    ) {
    }

}

pub trait Eventful {

    fn created(&mut self) {}

    fn flagged(&mut self, _: u8) {}

    fn destroyed(&mut self) {}

}

