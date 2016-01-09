use std::f32;
use arena::Arena;
use entity::{EntityType, EntityInput, EntityState, Entity};
use drawable::ZeroDrawable;

pub struct Ship {
    state: EntityState,
    base_state: EntityState,
    last_state: EntityState,
    max_speed: f32,
    acceleration: f32,
    rotation: f32,
    input_states: Vec<EntityInput>
}

impl EntityType for Ship {

    fn is_local(&self) -> bool {
        self.state.flags & 0x01 == 0x01
    }

    fn kind_id(&self) -> u8 {
        0
    }

    fn get_state(&mut self) -> EntityState  {
        self.state
    }

    fn set_state(&mut self, state: EntityState) {
        self.state = state;
        self.set_flags(state.flags);
        self.last_state = state;
        self.base_state = state;
    }

    fn interpolate_state(&self, arena: &Arena, u: f32) -> EntityState {
        arena.interpolate_state(&self.state, &self.last_state, u)
    }

    fn serialize_state(&self, _: &mut Vec<u8>) {

    }

    fn serialize_inputs(&self, _: &mut Vec<u8>) {

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

impl Ship {

    pub fn create_entity(scale: f32) -> Entity {
        Entity::new(
            Box::new(Ship::new(scale)),
            Box::new(ZeroDrawable)
        )
    }

    pub fn new(scale: f32) -> Ship {
        let state = EntityState {
            x: 0.0,
            y: 0.0,
            r: 0.0,
            mx: 0.0,
            my: 0.0,
            flags: 0
        };
        Ship {
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

