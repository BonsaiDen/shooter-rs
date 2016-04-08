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
        self.system.draw(dt, |ref particle| {

            let hs = particle.s / 2.0;
            let lp = 1.0 / particle.lifetime * particle.remaining;
            let alpha = if lp <= particle.fadeout {
                255.0 / (particle.lifetime * particle.fadeout) * particle.remaining.max(0.0)

            } else {
                255.0
            };

            prim.draw_filled_rectangle(
                particle.x - hs + 0.5, particle.y - hs + 0.5,
                particle.x + hs + 0.5, particle.y + hs + 0.5,
                AllegroRenderer::get_color_with_alpha(&particle.color, alpha as u8)
            );

        });
    }

}

