extern crate allegro;
extern crate allegro_primitives;

use std::cmp;
use self::allegro_primitives::PrimitivesAddon;
use color::RgbColor;

pub struct Particle {

    active: bool,

    pub color: RgbColor,

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

    fn draw(&mut self, core: &allegro::Core, prim: &PrimitivesAddon) {

        let lp = 1.0 / self.lifetime * self.remaining;
        let alpha = if lp <= self.fadeout {
            1.0 / (self.lifetime * self.fadeout) * self.remaining.max(0.0)

        } else {
            1.0
        };

        let hs = self.s / 2.0;
        prim.draw_filled_rectangle(
            self.x - hs + 0.5, self.y - hs + 0.5, self.x + hs + 0.5, self.y + hs + 0.5,
            self.color.to_rgba(alpha)
        );

    }

}

pub struct ParticleSystem {
    first_available_particle: usize,
    max_used_particle: usize,
    particles: Vec<Particle>
}

impl ParticleSystem {

    pub fn new(max_particles: usize) -> ParticleSystem {

        let mut particles = Vec::with_capacity(max_particles);
        for i in 0..max_particles {
            particles.push(Particle {
                active: false,
                color: RgbColor::black(),
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

    pub fn draw(&mut self, core: &allegro::Core, prim: &PrimitivesAddon, dt: f32) {

        let mut max_used_particle = 0;

        for i in 0..self.max_used_particle {
            let p = self.particles.get_mut(i).unwrap();
            if p.is_active() {

                if p.step(dt) == false {
                    p.next_available = self.first_available_particle;
                    self.first_available_particle = p.id;

                } else {
                    max_used_particle = cmp::max(p.id + 1, max_used_particle);
                    p.draw(core, prim);
                }
            }
        }

        self.max_used_particle = max_used_particle;

    }

}

