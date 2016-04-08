// External Dependencies ------------------------------------------------------



// Internal Dependencies ------------------------------------------------------
use renderer::{ParticleSystem, Particle};
use super::GliumRenderer;


// Glium based ParticleSystem -------------------------------------------------
pub struct GliumParticleSystem {
    system: ParticleSystem
}

impl GliumParticleSystem {

    pub fn new(max_particles: usize) -> GliumParticleSystem {
        GliumParticleSystem {
            system: ParticleSystem::new(max_particles)
        }
    }

    pub fn get(&mut self) -> Option<&mut Particle> {
        self.system.get()
    }

    pub fn draw(&mut self, dt: f32) {
        self.system.draw(dt, |ref particle| {
            // TODO update vertex buffer
        });
        // TODO draw vertex buffer
    }

}

