extern crate rand;
extern crate allegro;
extern crate allegro_primitives;

use self::allegro_primitives::PrimitivesAddon;
use self::rand::Rng;
use std::f32;

use color::Color;
use particle::ParticleSystem;

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
    drawable: DrawableShip,
    local: bool
}

impl PlayerShip {

    pub fn new(x: f32, y: f32, is_local: bool, color: Color) -> PlayerShip {
        PlayerShip {
            ship: Ship::new(x, y, 1.0),
            drawable: DrawableShip::new(color, 1.0),
            local: is_local
        }
    }

    pub fn is_local(&mut self) -> bool {
        self.local
    }

    pub fn input(&mut self, input: Input) {
        self.ship.input(input);
    }

    pub fn remote_step(&mut self, dt: f32, remote_tick: u8, state: ShipState) {
        self.ship.apply_remote_state(remote_tick, state);
        self.ship.apply_inputs(dt);
    }

    pub fn step(&mut self, dt: f32) {
        self.ship.apply_local_state();
        self.ship.apply_inputs(dt);
    }

    pub fn get_state(&mut self) -> ShipState  {
        self.ship.state
    }

    pub fn draw(
        &mut self, core: &allegro::Core, prim: &PrimitivesAddon,
        rng: &mut rand::XorShiftRng, dt: f32, u: f32
    ) {
        self.drawable.draw(
            core, prim, rng,
            &self.ship.state, &self.ship.last_state, dt, u
        );
    }

}

struct Ship {
    state: ShipState,
    base_state: ShipState,
    last_state: ShipState,
    max_speed: f32,
    acceleration: f32,
    rotation: f32,
    input_states: Vec<Input>
}

impl Ship {

    pub fn new(x: f32, y: f32, scale: f32) -> Ship {
        let state = ShipState {
            x: x,
            y: y,
            r: 0.0,
            mx: 0.0,
            my: 0.0,
            thrust: false
        };
        Ship {
            state: state,
            base_state: state,
            last_state: state,
            input_states: Vec::new(),
            max_speed: 90.0 * scale,
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

        self.last_state = self.state;
        self.base_state = state;
        self.state = state;

        // Drop all confirmed inputs
        self.input_states.retain(|input| {
            tick_is_more_recent(input.tick, remote_tick)
        });

    }

    pub fn apply_local_state(&mut self) {
        self.last_state = self.state;
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
            // Constant time acceleration
            let m = 60.0 / (1.0 / dt);
            state.mx += state.r.cos() * acceleration * dt * 60.0 / (1.0 / dt);
            state.my += state.r.sin() * acceleration * dt * m;
        }

        state.thrust = input.thrust;

        // Limit diagonal speed
        let mr = state.my.atan2(state.mx);
        let mut s = ((state.mx * state.mx) + (state.my * state.my)).sqrt();

        // Allow for easier full stop
        if s < 0.15 {
            s *= 0.95;
        }

        // Limit max speed
        state.mx = mr.cos() * s.min(max_speed * dt);
        state.my = mr.sin() * s.min(max_speed * dt);
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
    scale: f32,
    particle_system: ParticleSystem
}

impl DrawableShip {

    pub fn new(color: Color, scale: f32) -> DrawableShip {
        DrawableShip {
            color_light: color,
            color_mid: color.darken(0.5),
            scale: scale,
            particle_system: ParticleSystem::new(50)
        }
    }

    pub fn draw(
        &mut self, core: &allegro::Core, prim: &PrimitivesAddon,
        rng: &mut rand::XorShiftRng,
        state: &ShipState, last_state: &ShipState,
        dt: f32, u: f32
    ) {

        let mr = state.r - last_state.r;
        let draw_state = ShipState {
            r: last_state.r + mr.sin().atan2(mr.cos()) * u,
            x: last_state.x * (1.0 - u) + state.x * u,
            y: last_state.y * (1.0 - u) + state.y * u,
            mx: 0.0,
            my: 0.0,
            thrust: state.thrust
        };

        let light = self.color_light;
        let mid = self.color_mid;
        let scale = self.scale;

        self.draw_triangle(
            core, prim, &draw_state,
            mid, scale, scale, 1.15, -8.0, 6.0
        );
        self.draw_triangle(
            core, prim, &draw_state,
            light, scale, scale, (2 as f32).sqrt(), 12.0, 9.0
        );
        self.draw_triangle(
            core, prim, &draw_state,
            mid, scale, scale * 0.66, (2 as f32).sqrt(), 12.0, 9.0
        );

        if rng.gen::<u8>() > 20 && draw_state.thrust {
            if let Some(p) = self.particle_system.get() {

                // Exhaust angle
                let w = 0.95;
                let mr = draw_state.my.atan2(draw_state.mx);
                let d = draw_state.r - mr;

                // Increase engine velocity when flying backwards
                let mut dr = d.abs() % (f32::consts::PI * 2.0);
                if dr > f32::consts::PI  {
                    dr = f32::consts::PI * 2.0 - dr;
                }

                // Calculate exhaust angle
                let cs = (1.0 - w) * mr.cos() + w * draw_state.r.cos();
                let sn = (1.0 - w) * mr.sin() + w * draw_state.r.sin();
                let mr = sn.atan2(cs) + f32::consts::PI;

                // Spawn exhaust particles
                p.color = self.color_light;
                p.x = draw_state.x + mr.cos() * 9.0 * self.scale + 0.5;
                p.y = draw_state.y + mr.sin() * 9.0 * self.scale + 0.5;
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

        self.particle_system.draw(&core, &prim, dt);

    }

    fn draw_triangle(
        &self, core: &allegro::Core, prim: &PrimitivesAddon,
        state: &ShipState, color: Color,
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
        prim.draw_triangle(ax, ay, bx, by, cx, cy, color.map_rgb(core), 0.5 * body_scale);
    }

}

