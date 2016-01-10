// External Dependencies ------------------------------------------------------
use allegro;
use allegro_primitives::PrimitivesAddon;
use rand::XorShiftRng;
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
use entity;
use arena::Arena;
use particle::ParticleSystem;


// Basic Entity Traits --------------------------------------------------------
pub trait Base : Ticked + Stateful + Owned + Controlled + Eventful {

    fn typ(&self) -> u8;

}

pub trait Drawable : Eventful {

    fn draw(
        &mut self,
        _: &allegro::Core, _: &PrimitivesAddon,
        _: &mut XorShiftRng, _: &mut ParticleSystem,
        _: &Arena, _: &Base, _: f32, _: f32
    ) {
    }

}


// Behavior Implementation Traits ---------------------------------------------
pub trait Ticked {

    fn tick_local(&mut self, arena: &Arena, dt: f32, temporary: bool);

    fn tick_remote(
        &mut self,
        arena: &Arena,
        dt: f32, remote_tick: u8, state: entity::State
    );

}

pub trait Stateful {

    fn get_state(&self) -> entity::State;

    fn set_state(&mut self, state: entity::State);

    fn interpolate_state(&self, arena: &Arena, u: f32) -> entity::State;

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

