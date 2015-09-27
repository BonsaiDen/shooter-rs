use allegro;
use allegro_primitives::PrimitivesAddon;
use rand::XorShiftRng;

use arena::Arena;
use entity::Entity;

pub trait Drawable {

    fn draw(
        &mut self,
        core: &allegro::Core, prim: &PrimitivesAddon, rng: &mut XorShiftRng,
        arena: &Arena, entity: &Entity, dt: f32, u: f32
    );

}

