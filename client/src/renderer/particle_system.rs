// External Dependencies ------------------------------------------------------
use std::cmp;


// Internal Dependencies ------------------------------------------------------
use shared::{Color, ColorName};


// ParticleSystem -------------------------------------------------------------
pub struct ParticleSystem {
    pub first_available_particle: usize,
    pub max_used_particle: usize,
    pub particles: Vec<Particle>
}

impl ParticleSystem {

    pub fn new(max_particles: usize) -> ParticleSystem {

        let mut particles = Vec::with_capacity(max_particles);
        for i in 0..max_particles {
            particles.push(Particle {
                active: false,
                color: Color::from_name(ColorName::Black),
                x: 0.0,
                y: 0.0,
                s: 1.0,
                sms: 0.0,
                v: 0.0,
                vms: 0.0,
                r: 0.0,
                rms: 0.0,
                fadeout: 0.0,
                lifetime: 0.0,
                remaining: 0.0,
                id: i,
                next_available: i + 1,
            });
        }

        ParticleSystem {
            first_available_particle: 0,
            max_used_particle: 0,
            particles: particles
        }

    }

    pub fn get(&mut self) -> Option<&mut Particle> {

        if let Some(p) = self.particles.get_mut(self.first_available_particle) {
            p.active = true;
            p.x = 0.0;
            p.y = 0.0;
            p.s = 5.0;
            p.sms = -2.5;
            p.v = 0.0;
            p.vms = 0.0;
            p.r = 0.0;
            p.rms = 0.0;
            p.fadeout = 0.25;
            p.lifetime = 0.8;
            p.remaining = 0.8;
            self.first_available_particle = p.next_available;
            self.max_used_particle = cmp::max(self.max_used_particle, p.id + 1);
            Some(p)

        } else {
            None
        }

    }

    pub fn draw<F>(&mut self, dt: f32, draw_callback: F) where F: Fn(&Particle) {

        let mut max_used_particle = 0;

        for i in 0..self.max_used_particle {
            let particle = self.particles.get_mut(i).unwrap();
            if particle.is_active() {
                if particle.step(dt) == false {
                    particle.next_available = self.first_available_particle;
                    self.first_available_particle = particle.id;

                } else {
                    max_used_particle = cmp::max(
                        particle.id + 1,
                        max_used_particle
                    );
                }
                draw_callback(particle);
            }
        }

        self.max_used_particle = max_used_particle;

    }

}


// Particle -------------------------------------------------------------------
pub struct Particle {

    active: bool,

    pub color: Color,

    // Position
    pub x: f32,
    pub y: f32,

    // Velocity
    pub v: f32,

    // Size
    pub s: f32,

    // Size modification per second
    pub sms: f32,

    // Velocity modification per seond
    pub vms: f32,

    // Angle
    pub r: f32,

    // Angle modification per second
    pub rms: f32,

    pub fadeout: f32,
    pub lifetime: f32,
    pub remaining: f32,

    pub id: usize,
    pub next_available: usize

}

impl Particle {

    fn is_active(&mut self) -> bool {
        self.active
    }

    fn step(&mut self, dt: f32) -> bool {
        if self.remaining <= 0.0 {
            self.active = false;
            false

        } else {
            self.x += self.r.cos() * self.v * dt;
            self.y += self.r.sin() * self.v * dt;
            self.s += self.sms * dt;
            self.r += self.rms * dt;
            self.v += self.vms * dt;
            self.remaining -= dt;
            true
        }
    }

}

