extern crate rand;
extern crate allegro;
extern crate allegro_primitives;

use allegro_primitives::*;

use std::collections::VecDeque;
use particle::ParticleSystem;
use color::Color;
use std::f32;
use rand::Rng;

pub struct Input {
    pub tick: u8,
    pub left: bool,
    pub right: bool,
    pub thrust: bool,
    pub fire: bool
}

#[derive(Copy, Clone)]
pub struct ShipState {
    x: f32,
    y: f32,
    r: f32,
    mx: f32,
    my: f32,
    thrust: bool
}

pub struct PlayerShip {
    ship: Ship,
    drawable: DrawableShip
}

impl PlayerShip {

    pub fn new(x: f32, y: f32, color: Color) -> PlayerShip {
        PlayerShip {
            ship: Ship::new(x, y, 1.0),
            drawable: DrawableShip::new(x, y, color)
        }
    }

    pub fn input(&mut self, input: Input) {
        self.ship.input(input);
    }

    pub fn apply_remote_state(&mut self, remote_tick: u8, state: ShipState) {
        self.ship.apply_remote_state(remote_tick, state);
    }

    pub fn get_state(&mut self) -> ShipState  {
        self.ship.state
    }

    pub fn step(&mut self, rng: &mut rand::XorShiftRng, dt: f32) {
        self.ship.reset_state();
        // TODO use fixed DT
        self.ship.apply_inputs(dt);
        // TODO seed the rng with the tick and some other stuff?
        self.drawable.step(rng, &self.ship.state, dt);
    }

    pub fn draw(&mut self, core: &allegro::Core, prim: &PrimitivesAddon, dt: f32) {
        self.drawable.draw(core, prim, &self.ship, dt);
    }

}

struct Ship {
    state: ShipState,
    base_state: ShipState,
    max_speed: f32,
    acceleration: f32,
    rotation: f32,
    input_states: Vec<Input>
}

impl Ship {

    pub fn new(x: f32, y: f32, scale: f32) -> Ship {
        Ship {
            state: ShipState {
                x: x,
                y: y,
                r: 0.0,
                mx: 0.0,
                my: 0.0,
                thrust: false
            },
            base_state: ShipState {
                x: x,
                y: y,
                r: 0.0,
                mx: 0.0,
                my: 0.0,
                thrust: false
            },
            input_states: Vec::new(),
            max_speed: 1.5 * scale,
            acceleration: 2.0 * scale,
            rotation: 120.0
        }
    }

    pub fn input(&mut self, input: Input) {

        self.input_states.push(input);

        if self.input_states.len() > 30 {
            self.input_states.remove(0);
        }

    }

    pub fn apply_remote_state(&mut self, remote_tick: u8, state: ShipState) {

        self.base_state = state;
        self.state = state;

        // Drop all confirmed inputs
        self.input_states.retain(|input| {
            tick_is_more_recent(input.tick, remote_tick)
        });

    }

    pub fn reset_state(&mut self) {
        self.state = self.base_state;
    }

    pub fn apply_inputs(&mut self, dt: f32) {

        // Apply unconfirmed inputs on top of last state confirmed by the server
        let mut state = self.base_state;
        for input in self.input_states.iter() {
            Ship::apply_input_to_state(
                &input, &mut state, dt,
                self.rotation, self.acceleration, self.max_speed
            );
        }

        // Set new local state from replayed inputs
        self.state = state;

    }

    fn apply_input_to_state(
        input: &Input, state: &mut ShipState, dt: f32,
        rotation: f32,
        acceleration: f32,
        max_speed: f32
    ) {

        let mut steer = 0.0;
        if input.left {
            steer -= 1.0;
        }

        if input.right {
            steer += 1.0;
        }

        state.r += f32::consts::PI / 180.0 * rotation * dt * steer;

        if input.thrust {
            let ax = state.r.cos() * acceleration * dt;
            let ay = state.r.sin() * acceleration * dt;
            state.mx += ax;
            state.my += ay;
        }

        state.thrust = input.thrust;

        // Limit speed
        let mr = state.my.atan2(state.mx);
        let mut s = ((state.mx * state.mx) + (state.my * state.my)).sqrt();

        // Allow for easier full stop
        if s < 0.15 {
            s *= 0.95;
        }

        state.mx = mr.cos() * s.min(max_speed);
        state.my = mr.sin() * s.min(max_speed);
        state.x += state.mx;
        state.y += state.my;

    }

}

fn tick_is_more_recent(a: u8, b: u8) -> bool {
    (a > b) && (a - b <= 255 / 2) || (b > a) && (b - a > 255 / 2)
}

struct DrawableShip {
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
            color_light: color,
            color_mid: color.darken(0.5),
            color_dark: color.darken(0.75),
            scale: scale,
            particle_system: ParticleSystem::new(50)
        }
    }

    pub fn step(
        &mut self, rng: &mut rand::XorShiftRng, state: &ShipState, dt: f32
    ) {

        if rng.gen::<u8>() > 20 && state.thrust {
            if let Some(p) = self.particle_system.get() {

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

                // Spawn exhaust particles
                p.color = self.color_light;
                p.x = state.x + mr.cos() * 9.0 * self.scale + 0.5;
                p.y = state.y + mr.sin() * 9.0 * self.scale + 0.5;
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

    }

    pub fn draw(
        &mut self, core: &allegro::Core, prim: &PrimitivesAddon, ship: &Ship,
        dt: f32
    ) {

        let light = self.color_light;
        let mid = self.color_mid;
        let scale = self.scale;

        self.draw_triangle(core, prim, ship, mid, scale, scale, 1.15, -8.0, 6.0);
        self.draw_triangle(core, prim, ship, light, scale, scale, (2 as f32).sqrt(), 12.0, 9.0);
        self.draw_triangle(core, prim, ship, mid, scale, scale * 0.66, (2 as f32).sqrt(), 12.0, 9.0);

        self.particle_system.draw(&core, &prim, dt);

    }

    fn draw_triangle(
        &self, core: &allegro::Core, prim: &PrimitivesAddon, ship: &Ship,
        color: Color, base_scale: f32, body_scale: f32, dr: f32, da: f32, db: f32
    ) {
        let beta = f32::consts::PI / dr;
        let ox = ship.state.r.cos() * -2.0 * base_scale + 0.5;
        let oy = ship.state.r.sin() * -2.0 * base_scale + 0.5;
        let ax = ox + ship.state.x + ship.state.r.cos() * da * body_scale;
        let ay = oy + ship.state.y + ship.state.r.sin() * da * body_scale;
        let bx = ox + ship.state.x + (ship.state.r + beta).cos() * db * body_scale;
        let by = oy + ship.state.y + (ship.state.r + beta).sin() * db * body_scale;
        let cx = ox + ship.state.x + (ship.state.r - beta).cos() * db * body_scale;
        let cy = oy + ship.state.y + (ship.state.r - beta).sin() * db * body_scale;
        prim.draw_triangle(ax, ay, bx, by, cx, cy, color.map_rgb(core), 0.5 * body_scale);
    }

}

