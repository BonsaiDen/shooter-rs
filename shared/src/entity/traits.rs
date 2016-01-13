// External Dependencies ------------------------------------------------------
use rand::XorShiftRng;
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
use entity;
use arena::Arena;
use renderer::Renderer;


// Basic Entity Traits --------------------------------------------------------
pub trait Base : Stateful + Owned + Controlled + Eventful {

    fn typ(&self) -> u8;

}

pub trait Drawable : Eventful {

    fn draw(
        &mut self,
        _: &mut Renderer,
        _: &mut XorShiftRng,
        _: &Arena, _: &Base, _: f32, _: f32
    ) {
    }

}


// Behavior Implementation Traits ---------------------------------------------
pub trait Stateful {

    fn tick(&mut self, arena: &Arena, dt: f32, server: bool);

    fn get_state(&self) -> entity::State;

    fn set_state(&mut self, state: entity::State, override_last: bool);

    fn interpolate_state(&self, arena: &Arena, u: f32) -> entity::State;

    fn set_remote_state(&mut self, tick: u8, state: entity::State);

}

pub trait Owned {

    fn visible_to(&self, _: &ConnectionID) -> bool {
        true
    }

}

pub trait Controlled {

    fn local(&self) -> bool;

    fn pending_inputs(&self) -> &Vec<entity::Input>;

    fn input(&mut self, input: entity::Input, max_inputs: usize);

}

pub trait Eventful {

    fn created(&mut self) {}

    fn flagged(&mut self, _: u8) {}

    fn destroyed(&mut self) {}

}

