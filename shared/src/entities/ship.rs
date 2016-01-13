// External Dependencies ------------------------------------------------------
use std::f32;


// Internal Dependencies ------------------------------------------------------
use entity;
use entity::traits::Eventful;
use arena::Arena;


// Ship Logic Implementation --------------------------------------------------
pub struct Ship {
    state: entity::State,
    base_state: entity::State,
    last_state: entity::State,
    remote_state: Option<(u8, entity::State)>,
    max_speed: f32,
    acceleration: f32,
    rotation: f32,
    input_states: Vec<entity::Input>,
    last_input_tick: u8
}

impl Ship {

    pub fn create_entity(scale: f32) -> entity::Entity {
        entity::Entity::new(
            Box::new(Ship::new(scale)),
            Box::new(ZeroDrawable)
        )
    }

    pub fn new(scale: f32) -> Ship {
        let state = entity::State {
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
            remote_state: None,
            max_speed: 90.0 * scale,
            acceleration: 2.0 * scale,
            rotation: 120.0,
            input_states: Vec::new(),
            last_input_tick: 0
        }
    }

}


// Trait Implementations ------------------------------------------------------
impl entity::traits::Base for Ship {

    fn typ(&self) -> u8 {
        0
    }

}

impl entity::traits::Owned for Ship {}
impl entity::traits::Eventful for Ship {}

impl entity::traits::Controlled for Ship {

    fn local(&self) -> bool {
        self.state.flags & 0x01 == 0x01
    }

    fn pending_inputs(&self) -> &Vec<entity::Input> {
        &self.input_states
    }

    fn input(&mut self, input: entity::Input, max_inputs: usize) {

        // Ignore inputs for past ticks
        if self.input_states.len() == 0 || entity::tick_is_more_recent(
            input.tick,
            self.last_input_tick
        ) {
            self.input_states.push(input);
            self.last_input_tick = input.tick;
        }

        // Drop outdated inputs
        if self.input_states.len() > max_inputs {
            self.input_states.remove(0);
        }

    }

}

impl entity::traits::Stateful for Ship {

    fn tick(&mut self, arena: &Arena, dt: f32, server: bool) {

        // Check if we have a remote state
        if let Some((remote_tick, remote_state)) = self.remote_state.take() {

            // Set the current state as the last state
            self.last_state = self.state;

            // Take over the remote state as the new base
            self.base_state = remote_state;
            self.state = remote_state;

            // Drop all inputs confirmed by the remote so the remaining ones
            // get applied on top of the new base state
            self.input_states.retain(|input| {
                entity::tick_is_more_recent(input.tick, remote_tick)
            });

        // Otherwise reset the local state and re-apply the inputs on top of it
        } else {
            self.last_state = self.state;
            self.state = self.base_state;
        }

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

        // Use the newly calculated state as the base
        if server {
            self.base_state = self.state;
            self.input_states.clear();
        }

    }

    fn get_state(&self) -> entity::State  {
        self.state
    }

    fn set_state(&mut self, state: entity::State, override_last: bool) {
        self.last_state = if override_last {
            state

        } else {
            self.state
        };
        self.base_state = state;
        self.state = state;
        self.flagged(state.flags);
    }

    fn interpolate_state(&self, arena: &Arena, u: f32) -> entity::State {
        arena.interpolate_state(&self.state, &self.last_state, u)
    }

    fn set_remote_state(&mut self, tick: u8, state: entity::State) {
        self.remote_state = Some((tick, state));
    }

}


// Noop Drawable --------------------------------------------------------------
struct ZeroDrawable;
impl entity::traits::Drawable for ZeroDrawable {}
impl entity::traits::Eventful for ZeroDrawable {}


// Input / Movement Logic -----------------------------------------------------
fn apply_input_to_state(
    input: &entity::Input, state: &mut entity::State, dt: f32,
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
        state.mx += state.r.cos() * acceleration * dt * m;
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

