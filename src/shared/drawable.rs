use allegro;
use allegro_primitives::PrimitivesAddon;
use rand::XorShiftRng;

use arena::Arena;
use entity::Entity;

pub trait Drawable {

    fn create(&mut self) {
    }

    fn flags(&mut self, old: u8, new: u8) {
    }

    fn destroy(&mut self) {
    }

    // TODO need one global particle system
    fn draw(
        &mut self,
        core: &allegro::Core, prim: &PrimitivesAddon, rng: &mut XorShiftRng,
        arena: &Arena, entity: &Entity, dt: f32, u: f32
    );

}

