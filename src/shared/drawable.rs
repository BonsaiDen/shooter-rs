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
        _: &allegro::Core, _: &PrimitivesAddon,
        _: &mut XorShiftRng, _: &mut ParticleSystem,
        _: &Arena, _: &EntityType, _: f32, _: f32
    ) {
    }

}

pub struct ZeroDrawable;
impl Drawable for ZeroDrawable {

}

