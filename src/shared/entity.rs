use allegro;
use allegro_primitives::PrimitivesAddon;
use rand::XorShiftRng;

use arena::Arena;

pub struct EntityInput {
    pub tick: u8,
    pub left: bool,
    pub right: bool,
    pub thrust: bool,
    pub fire: bool
}

#[derive(Copy, Clone)]
pub struct EntityState {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub mx: f32,
    pub my: f32,
    pub flags: u8
}

pub trait Entity {

    fn is_local(&self) -> bool {
        false
    }

    fn get_state(&mut self) -> EntityState;

    fn input(&mut self, input: EntityInput);

    fn tick(&mut self, arena: &Arena, dt: f32, set_last_state: bool);

    fn remote_tick(
        &mut self,
        arena: &Arena,
        dt: f32, set_last_state: bool, remote_tick: u8, state: EntityState
    );

    fn draw(
        &mut self,
        core: &allegro::Core, prim: &PrimitivesAddon, rng: &mut XorShiftRng,
        arena: &Arena, dt: f32, u: f32
    );

}

