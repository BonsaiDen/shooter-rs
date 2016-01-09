use allegro;
use allegro_primitives::PrimitivesAddon;

use std::{cmp, f32};
use rand::{Rng, XorShiftRng};

use shared::entities;
use shared::arena::Arena;
use shared::entity::{Entity, EntityType, EntityState};
use shared::drawable::Drawable;
use shared::color::{Color, RgbColor};
use shared::particle::ParticleSystem;

pub struct Ship {
    color_light: RgbColor,
    color_mid: RgbColor,
    scale: f32,
    particle_count: u32
}

impl Drawable for Ship {

    fn set_flags(&mut self, flags: u8) {
        self.color_light = Color::from_flags(flags).to_rgb();
        self.color_mid = self.color_light.darken(0.5);
    }

    fn draw(
        &mut self,
        core: &allegro::Core, prim: &PrimitivesAddon,
        rng: &mut XorShiftRng, particle_system: &mut ParticleSystem,
        arena: &Arena, entity: &EntityType, dt: f32, u: f32
    ) {

        let light = self.color_light;
        let mid = self.color_mid;
        let scale = self.scale;

        let state = entity.interpolate_state(arena, u);

        // Rendering
        self.draw_triangle(
            core, prim, &state,
            mid, scale, scale, 1.15, -9.0, 6.0
        );
        self.draw_triangle(
            core, prim, &state,
            light, scale, scale, (2 as f32).sqrt(), 12.0, 9.0
        );
        self.draw_triangle(
            core, prim, &state,
            mid, scale, scale * 0.66, (2 as f32).sqrt(), 12.0, 9.0
        );

        // Effects
        if state.flags & 0x02 == 0x02 {

            if rng.gen::<u8>() > 50 || self.particle_count > 1 {

                // Exhause more particles initially
                for _ in 0..self.particle_count {

                    if let Some(p) = particle_system.get() {

                        // Exhaust angle
                        let w = 0.95;
                        let mr = state.my.atan2(state.mx);
                        let d = state.r - mr;

                        // Increase engine velocity when flying backwards
                        let mut dr = d.abs() % (f32::consts::PI * 2.0);
                        if dr > f32::consts::PI  {
                            dr = f32::consts::PI * 2.0 - dr;
                        }

                        // Calculate exhaust angle
                        let cs = (1.0 - w) * mr.cos() + w * state.r.cos();
                        let sn = (1.0 - w) * mr.sin() + w * state.r.sin();
                        let mr = sn.atan2(cs) + f32::consts::PI;
                        let ar = ((rng.gen::<u8>() as f32) / 255.0 - 0.5) * (f32::consts::PI * 0.65);

                        // Spawn exhaust particles
                        p.color = self.color_light;
                        p.x = state.x + mr.cos() * 9.0 * self.scale + 0.5;
                        p.y = state.y + mr.sin() * 9.0 * self.scale + 0.5;
                        p.s = 2.5 * self.scale;
                        p.sms = -1.25 * self.scale;
                        p.v = ((86.0 + rng.gen::<u8>() as f32 / 9.0) * 0.5 + dr * 30.0) * 0.5 * self.scale;
                        p.vms = 0.0;
                        p.r = mr - ar * 1.7;
                        // Spread out exhaust
                        p.rms = ar * 1.25;

                        p.fadeout = 0.35;
                        p.lifetime = 0.4;
                        p.remaining = p.lifetime;

                    }
                }

            }

            self.particle_count = 1;

        } else {
            self.particle_count = cmp::min(self.particle_count + 1, 5);
        }

    }

}

impl Ship {

    pub fn create_entity(scale: f32) -> Entity {
        Entity::new(
            Box::new(entities::Ship::new(scale)),
            Box::new(Ship::new(scale))
        )
    }

    pub fn new(scale: f32) -> Ship {
        Ship {
            color_light: Color::Grey.to_rgb(),
            color_mid: Color::Grey.to_rgb().darken(0.5),
            scale: scale,
            particle_count: 5
        }
    }

    fn draw_triangle(
        &self, _: &allegro::Core, prim: &PrimitivesAddon,
        state: &EntityState, color: RgbColor,
        base_scale: f32, body_scale: f32, dr: f32, da: f32, db: f32
    ) {
        let beta = f32::consts::PI / dr;
        let ox = state.r.cos() * -2.0 * base_scale + 0.5;
        let oy = state.r.sin() * -2.0 * base_scale + 0.5;
        let ax = ox + state.x + state.r.cos() * da * body_scale;
        let ay = oy + state.y + state.r.sin() * da * body_scale;
        let bx = ox + state.x + (state.r + beta).cos() * db * body_scale;
        let by = oy + state.y + (state.r + beta).sin() * db * body_scale;
        let cx = ox + state.x + (state.r - beta).cos() * db * body_scale;
        let cy = oy + state.y + (state.r - beta).sin() * db * body_scale;
        prim.draw_triangle(ax, ay, bx, by, cx, cy, color.to_rgb(), 0.5 * body_scale);
    }

}

