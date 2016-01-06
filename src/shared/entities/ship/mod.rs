use std::f32;
use std::cmp;
use rand::{Rng, XorShiftRng};

use allegro;
use allegro_primitives::PrimitivesAddon;

use arena::Arena;
use color::{Color, RgbColor};
use entity::{EntityType, EntityInput, EntityState, Entity};
use drawable::{Drawable, ZeroDrawable};
use particle::ParticleSystem;

pub fn Ship(scale: f32) -> Entity {
    Entity::new(
        Box::new(ShipEntity::new(scale)),
        Box::new(ZeroDrawable)
    )
}

pub fn DrawableShip(scale: f32) -> Entity {
    Entity::new(
        Box::new(ShipEntity::new(scale)),
        Box::new(ShipDrawable::new(scale))
    )
}

pub struct ShipEntity {
    id: u32,
    state: EntityState,
    base_state: EntityState,
    last_state: EntityState,
    max_speed: f32,
    acceleration: f32,
    rotation: f32,
    input_states: Vec<EntityInput>
}

impl EntityType for ShipEntity {

    fn is_local(&self) -> bool {
        self.state.flags & 0x01 == 0x01
    }

    fn kind_id(&self) -> u8 {
        0
    }

    fn get_id(&self) -> u32 {
        self.id
    }

    fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    fn get_state(&mut self) -> EntityState  {
        self.state
    }

    fn set_state(&mut self, state: EntityState) {
        let old_flags = self.state.flags;
        self.state = state;
        self.set_flags(old_flags, state.flags);
        self.last_state = state;
        self.base_state = state;
    }

    fn interpolate_state(&self, arena: &Arena, u: f32) -> EntityState {
        arena.interpolate_state(&self.state, &self.last_state, u)
    }

    fn serialize_state(&self, buffer: &mut Vec<u8>) {

    }

    fn serialize_inputs(&self, buffer: &mut Vec<u8>) {

    }

    fn input(&mut self, input: EntityInput) {

        self.input_states.push(input);

        if self.input_states.len() > 30 {
            self.input_states.remove(0);
        }

    }

    fn remote_tick(
        &mut self,
        arena: &Arena,
        dt: f32, remote_tick: u8, state: EntityState
    ) {
        self.apply_remote_state(remote_tick, state);
        self.apply_inputs(arena, dt);
    }

    fn tick(&mut self, arena: &Arena, dt: f32) {
        self.apply_local_state();
        self.apply_inputs(arena, dt);
    }

}

impl ShipEntity {

    pub fn new(scale: f32) -> ShipEntity {
        let state = EntityState {
            x: 0.0,
            y: 0.0,
            r: 0.0,
            mx: 0.0,
            my: 0.0,
            flags: 0
        };
        ShipEntity {
            id: 0,
            state: state,
            base_state: state,
            last_state: state,
            input_states: Vec::new(),
            max_speed: 90.0 * scale,
            acceleration: 2.0 * scale,
            rotation: 120.0,
        }
    }

    pub fn apply_remote_state(
        &mut self, remote_tick: u8, state: EntityState
    ) {

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

    pub fn apply_inputs(&mut self, arena: &Arena, dt: f32) {

        // Apply unconfirmed inputs on top of last state confirmed by the server
        let mut state = self.base_state;
        for input in self.input_states.iter() {
            apply_input_to_state(
                &input, &mut state, dt,
                self.rotation, self.acceleration, self.max_speed
            );
        }

        // Set new local state from replayed inputs
        self.state = state;

        // Handle state wrapping
        arena.wrap_state(&mut self.state);

    }

}

fn apply_input_to_state(
    input: &EntityInput, state: &mut EntityState, dt: f32,
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
        state.flags |= 0x02;

    } else {
        state.flags &= !0x02;
    }

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

fn tick_is_more_recent(a: u8, b: u8) -> bool {
    (a > b) && (a - b <= 255 / 2) || (b > a) && (b - a > 255 / 2)
}


pub struct ShipDrawable {
    color_light: RgbColor,
    color_mid: RgbColor,
    scale: f32,
    particle_count: u32
}

impl Drawable for ShipDrawable {

    fn set_flags(&mut self, old: u8, new: u8) {
        self.color_light = Color::from_flags(new).to_rgb();
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

impl ShipDrawable {

    pub fn new(scale: f32) -> ShipDrawable {
        ShipDrawable {
            color_light: Color::Grey.to_rgb(),
            color_mid: Color::Grey.to_rgb().darken(0.5),
            scale: scale,
            particle_count: 5
        }
    }

    fn draw_triangle(
        &self, core: &allegro::Core, prim: &PrimitivesAddon,
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
        prim.draw_triangle(ax, ay, bx, by, cx, cy, color.map_rgb(core), 0.5 * body_scale);
    }

}

