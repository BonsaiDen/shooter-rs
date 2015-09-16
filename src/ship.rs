extern crate rand;
extern crate allegro;
extern crate allegro_primitives;

use allegro_primitives::*;

use particle::ParticleSystem;
use color::Color;
use std::f32;
use rand::Rng;

struct Ship {
    x: f32,
    y: f32,
    r: f32,
    mx: f32,
    my: f32,
    max_speed: f32,
    acceleration: f32,
    rotation: f32
}

impl Ship {

    pub fn new(x: f32, y: f32, scale: f32) -> Ship {
        Ship {
            x: x,
            y: y,
            r: 0.0,
            mx: 0.0,
            my: 0.0,
            max_speed: 1.5 * scale,
            acceleration: 2.0 * scale,
            rotation: 120.0
        }
    }

    pub fn step(&mut self, dt: f32, mut steer: f32, thrust: bool, jr: f32) {

        // TODO Test out gamepad controls
        //if jr != 255.0 {

        //    let jd = (self.r - jr).sin().atan2((self.r - jr).cos());
        //    if jd < 0.0 {
        //        steer = 1.0;

        //    } else if jd > 0.0 {
        //        steer = -1.0;
        //    }

        //}

        self.r += f32::consts::PI / 180.0 * self.rotation * dt * steer;

        if thrust {
            let ax = self.r.cos() * self.acceleration * dt;
            let ay = self.r.sin() * self.acceleration * dt;
            self.mx += ax;
            self.my += ay;
        }


        // Limit speed
        let mr = self.my.atan2(self.mx);
        let mut s = ((self.mx * self.mx) + (self.my * self.my)).sqrt();

        // Allow for easier full stop
        if s < 0.15 {
            s *= 0.95;
        }

        self.mx = mr.cos() * s.min(self.max_speed);
        self.my = mr.sin() * s.min(self.max_speed);
        self.x += self.mx;
        self.y += self.my;


    }

}

pub struct DrawableShip {
    ship: Ship,
    color_light: Color,
    color_mid: Color,
    color_dark: Color,
    scale: f32,
    particle_system: ParticleSystem
}

impl DrawableShip {

    pub fn new(x: f32, y: f32, color: Color) -> DrawableShip {
        let scale = 1.0;
        let ship = Ship::new(x, y, scale);
        DrawableShip {
            ship: ship,
            color_light: color,
            color_mid: color.darken(0.5),
            color_dark: color.darken(0.75),
            scale: scale,
            particle_system: ParticleSystem::new(50)
        }
    }

    pub fn step(&mut self, rng: &mut rand::XorShiftRng, dt: f32, steer: f32, thrust: bool, jr: f32) {

        if rng.gen::<u8>() > 20 && thrust {
            if let Some(p) = self.particle_system.get() {

                // Exhaust angle
                let w = 0.95;
                let mr = self.ship.my.atan2(self.ship.mx);
                let d = self.ship.r - mr;

                // Increase engine velocity when flying backwards
                let mut dr = d.abs() % (f32::consts::PI * 2.0);
                if dr > f32::consts::PI  {
                    dr = f32::consts::PI * 2.0 - dr;
                }

                // Calculate exhaust angle
                let cs = (1.0 - w) * mr.cos() + w * self.ship.r.cos();
                let sn = (1.0 - w) * mr.sin() + w * self.ship.r.sin();
                let mr = sn.atan2(cs) + f32::consts::PI;

                // Spawn exhaust particles
                p.color = self.color_light; //COLORS[self.ship.color as usize];
                p.x = self.ship.x + mr.cos() * 9.0 * self.scale + 0.5;
                p.y = self.ship.y + mr.sin() * 9.0 * self.scale + 0.5;
                p.s = 2.5 * self.scale;
                p.sms = -1.25 * self.scale;
                p.v = ((86.0 + rng.gen::<u8>() as f32 / 9.0) * 0.5 + dr * 30.0) * 0.5 * self.scale;
                p.vms = 0.0;
                p.r = mr + ((rng.gen::<u8>() as f32) - 96.0) / 96.0;
                p.rms = ((rng.gen::<u8>() as f32) - 128.0) / 128.0;
                p.fadeout = 0.25;
                p.lifetime = 0.5;
                p.remaining = 0.5;

            }
        }

        self.ship.step(dt, steer, thrust, jr);

    }

    pub fn draw(&mut self, core: &allegro::Core, prim: &PrimitivesAddon, dt: f32) {

        let light = self.color_light;
        let mid = self.color_mid;
        let scale = self.scale;

        self.draw_triangle(core, prim, mid, scale, scale, 1.15, -8.0, 6.0);
        self.draw_triangle(core, prim, light, scale, scale, (2 as f32).sqrt(), 12.0, 9.0);
        self.draw_triangle(core, prim, mid, scale, scale * 0.66, (2 as f32).sqrt(), 12.0, 9.0);
        //prim.draw_circle(self.ship.x + 0.5, self.ship.y + 0.5, 11.0 * scale, self.color_dark.map_rgb(core), 0.5 * scale);

        self.particle_system.draw(&core, &prim, dt);

        // Velocity vector
        /*
        let mr = self.ship.my.atan2(self.ship.mx);
        let s = ((self.ship.mx * self.ship.mx) + (self.ship.my * self.ship.my)).sqrt() * 10.0;
        let tx = self.ship.x + mr.cos() * s;
        let ty = self.ship.y + mr.sin() * s;
        prim.draw_line(self.ship.x + 0.5, self.ship.y + 0.5, tx + 0.5, ty + 0.5, light, 1.0);

        //
        let w = 0.5;
        let mr = self.ship.my.atan2(self.ship.mx);
        let cs = (1.0 - w) * mr.cos() + w * self.ship.r.cos();
        let sn = (1.0 - w) * mr.sin() + w * self.ship.r.sin();
        let mc = sn.atan2(cs) + f32::consts::PI;
        let s = 50.0;
        let tx = self.ship.x + mc.cos() * s;
        let ty = self.ship.y + mc.sin() * s;
        prim.draw_line(self.ship.x + 0.5, self.ship.y + 0.5, tx + 0.5, ty + 0.5, light, 1.0);
        */
    }

    fn draw_triangle(&self, core: &allegro::Core, prim: &PrimitivesAddon, color: Color, base_scale: f32, body_scale: f32, dr: f32, da: f32, db: f32) {
        let beta = f32::consts::PI / dr;
        let ox = self.ship.r.cos() * -2.0 * base_scale + 0.5;
        let oy = self.ship.r.sin() * -2.0 * base_scale + 0.5;
        let ax = ox + self.ship.x + self.ship.r.cos() * da * body_scale;
        let ay = oy + self.ship.y + self.ship.r.sin() * da * body_scale;
        let bx = ox + self.ship.x + (self.ship.r + beta).cos() * db * body_scale;
        let by = oy + self.ship.y + (self.ship.r + beta).sin() * db * body_scale;
        let cx = ox + self.ship.x + (self.ship.r - beta).cos() * db * body_scale;
        let cy = oy + self.ship.y + (self.ship.r - beta).sin() * db * body_scale;
        prim.draw_triangle(ax, ay, bx, by, cx, cy, color.map_rgb(core), 0.5 * body_scale);
    }

}

