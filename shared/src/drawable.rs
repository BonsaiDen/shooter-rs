// External Dependencies ------------------------------------------------------
use allegro;
use allegro_primitives::PrimitivesAddon;
use rand::XorShiftRng;


// Internal Dependencies ------------------------------------------------------
use arena::Arena;
use entity;
use particle::ParticleSystem;


// Drawable Trait -------------------------------------------------------------
pub trait Drawable {

    fn set_flags(&mut self, _: u8) {
    }

    fn draw(
        &mut self,
        _: &allegro::Core, _: &PrimitivesAddon,
        _: &mut XorShiftRng, _: &mut ParticleSystem,
        _: &Arena, _: &entity::Kind, _: f32, _: f32
    ) {
    }

    fn create(&mut self) {
    }

    fn destroy(&mut self) {
    }

}


// Abstract Zero Drawable for Server Side Logic -------------------------------
pub struct ZeroDrawable;
impl Drawable for ZeroDrawable {}

