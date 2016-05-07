// External Dependencies ------------------------------------------------------
use allegro_primitives::PrimitivesAddon;


// Internal Dependencies ------------------------------------------------------
use renderer::{ParticleSystem, Particle};
use super::AllegroRenderer;


// Allegro based ParticleSystem -----------------------------------------------
pub struct AllegroParticleSystem {
    system: ParticleSystem
}

impl AllegroParticleSystem {

    pub fn new(max_particles: usize) -> AllegroParticleSystem {
        AllegroParticleSystem {
            system: ParticleSystem::new(max_particles)
        }
    }

    pub fn get(&mut self) -> Option<&mut Particle> {
        self.system.get()
    }

    pub fn draw(&mut self, dt: f32, prim: &PrimitivesAddon) {
        self.system.draw(dt, |_, ref particle, alpha| {
            let hs = particle.s / 2.0;
            prim.draw_filled_rectangle(
                particle.x - hs + 0.5, particle.y - hs + 0.5,
                particle.x + hs + 0.5, particle.y + hs + 0.5,
                AllegroRenderer::get_color_with_alpha(&particle.color, alpha as u8)
            );
        });
    }

}

