use allegro;
use allegro_primitives::PrimitivesAddon;
use rand::XorShiftRng;

use arena::Arena;
use entity::EntityType;
use particle::ParticleSystem;

pub trait Drawable {

    fn create(&mut self) {
    }

    fn set_flags(&mut self, old: u8, new: u8) {
    }

    fn destroy(&mut self) {
    }

    fn draw(
        &mut self,
        core: &allegro::Core, prim: &PrimitivesAddon,
        rng: &mut XorShiftRng, particle_system: &mut ParticleSystem,
        arena: &Arena, entity: &EntityType, dt: f32, u: f32
    ) {
    }

}

pub struct ZeroDrawable;
impl Drawable for ZeroDrawable {

}

