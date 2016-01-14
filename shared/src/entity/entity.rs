// External Dependencies ------------------------------------------------------
use rand::XorShiftRng;
use cobalt::ConnectionID;
use std::collections::VecDeque;


// Internal Dependencies ------------------------------------------------------
use entity;
use level::Level;
use renderer::Renderer;
use entity::traits::{Base, Drawable};


// Top Level Entity Structure -------------------------------------------------
pub struct Entity {

    entity: Box<Base>,
    drawable: Box<Drawable>,
    owner: Option<ConnectionID>,
    is_alive: bool,
    local_id: u16,

    // State
    state: entity::State,
    base_state: entity::State,
    last_state: entity::State,
    confirmed_state: Option<(u8, entity::State)>,
    state_buffer: VecDeque<(u8, entity::State)>,

    // Inputs
    input_buffer: VecDeque<entity::Input>,
    confirmed_input_tick: u8,
    initial_input: bool,

    // Configuration
    input_buffer_size: usize,
    state_buffer_size: usize

}

impl Entity {

    pub fn new(entity: Box<Base>, drawable: Box<Drawable>) -> Entity {
        Entity {

            // Entity Behavior
            entity: entity,

            // Entity Rendering
            drawable: drawable,

            // Owner of the Entity
            owner: None,

            // Whether the entity is still alive or should be destroyed
            is_alive: false,

            // Locally used Entity ID
            local_id: 0,

            // Current - calculated - entity state
            state: entity::State::default(),

            // Current base state (before apply pending inputs)
            base_state: entity::State::default(),

            // Previously caluclated state for interpolation purposes
            last_state: entity::State::default(),

            // Last confirmed remote state (client only)
            confirmed_state: None,

            // List of previous entity states for client-side interpolation
            // and server-side latency compensation
            state_buffer: VecDeque::new(),

            // Pending inputs (client only)
            input_buffer: VecDeque::new(),

            // Last tick for which input was received (server only)
            confirmed_input_tick: 0,
            initial_input: true,

            // Configuration TODO allow external configuration
            input_buffer_size: 30,
            state_buffer_size: 30 // length is tickDT * size

        }
    }


    // Getter / Setter --------------------------------------------------------
    pub fn id(&self) -> u16 {
        self.local_id
    }

    pub fn set_id(&mut self, id: u16) {
        self.local_id = id;
    }

    pub fn local(&self) -> bool {
        self.state.flags & 0x01 == 0x01
    }

    pub fn alive(&self) -> bool {
        self.is_alive
    }

    pub fn set_alive(&mut self, alive: bool) {
        self.is_alive = alive;
    }


    // Ownership --------------------------------------------------------------
    pub fn owner(&self) -> Option<&ConnectionID> {
        self.owner.as_ref()
    }

    pub fn set_owner(&mut self, owner: ConnectionID) {
        self.owner = Some(owner);
    }

    pub fn owned_by(&self, owner: &ConnectionID) -> bool {
        match self.owner {
            Some(o) => o == *owner,
            None => false
        }
    }

    pub fn visible_to(&self, owner: &ConnectionID) -> bool {
        self.entity.visible_to(owner)
    }


    // State ------------------------------------------------------------------
    pub fn get_state(&self) -> entity::State {
        self.state
    }

    pub fn offset_state(&self, tick_offset: usize) -> entity::State {
        let buffer_len = self.state_buffer.len();
        if buffer_len > 0 && tick_offset < buffer_len {
            self.state_buffer[tick_offset].1

        } else {
            self.state
        }
    }

    pub fn offset_state_pair(
        &self, tick_offset: usize

    ) -> (entity::State, entity::State) {
        let buffer_len = self.state_buffer.len();
        if buffer_len > 0 && tick_offset + 1 < buffer_len {
            (
                self.state_buffer[tick_offset].1,
                self.state_buffer[tick_offset + 1].1
            )
        } else {
            (
                self.state,
                self.last_state
            )
        }
    }

    pub fn set_state(&mut self, state: entity::State) {
        self.set_entity_state(state, true);
    }

    pub fn set_remote_state(&mut self, state: entity::State) {
        self.set_entity_state(state, false);
    }

    pub fn set_confirmed_state(&mut self, tick: u8, state: entity::State) {
        self.confirmed_state = Some((tick, state));
    }

    fn set_entity_state(&mut self, state: entity::State, override_last: bool) {

        let old_flags = self.state.flags;
        self.last_state = if override_last {
            state

        } else {
            self.state
        };

        self.base_state = state;
        self.state = state;

        if old_flags != state.flags {
            self.drawable.event_flags(state.flags);
            self.entity.event_flags(state.flags);
        }

    }


    // Input ------------------------------------------------------------------
    pub fn confirmed_tick(&self) -> u8 {
        self.confirmed_input_tick
    }

    pub fn local_input(&mut self, input: entity::Input) -> Vec<u8> {

        self.input(input);

        let mut inputs = Vec::new();
        for input in self.input_buffer.iter() {
            inputs.extend(input.serialize());
        }
        inputs

    }

    pub fn remote_input(&mut self, input: entity::Input) {

        if self.initial_input ||  tick_is_more_recent(
            input.tick,
            self.confirmed_input_tick
        ) {
            self.initial_input = false;
            self.confirmed_input_tick = input.tick;
            self.input(input);
        }

    }

    fn input(&mut self, input: entity::Input) {

        self.input_buffer.push_back(input);

        // Drop outdated inputs
        if self.input_buffer.len() > self.input_buffer_size {
            self.input_buffer.pop_front();
        }

    }


    // Ticking ----------------------------------------------------------------
    pub fn client_tick(&mut self, level: &Level, tick: u8, dt: f32) {
        self.entity.client_event_tick(level, &self.state, tick, dt);
        self.tick(level, tick, dt, false);
    }

    pub fn server_tick(&mut self,  level: &Level, tick: u8, dt: f32) {
        self.entity.server_event_tick(level, &self.state, tick, dt);
        self.tick(level, tick, dt, true);
    }

    fn tick(&mut self, level: &Level, tick: u8, dt: f32, server: bool) {

        // Check if we have a remote state
        if let Some((confirmed_tick, confirmed_state)) = self.confirmed_state.take() {

            // Set the current state as the last state
            self.last_state = self.state;

            // Take over the remote state as the new base
            self.base_state = confirmed_state;
            self.state = confirmed_state;

            // Drop all inputs confirmed by the remote so the remaining ones
            // get applied on top of the new base state
            self.input_buffer.retain(|input| {
                tick_is_more_recent(input.tick, confirmed_tick)
            });

        // Otherwise reset the local state and re-apply the inputs on top of it
        } else {
            self.last_state = self.state;
            self.state = self.base_state;
        }

        // Apply unconfirmed inputs on top of last state confirmed by the server
        let mut new_state = self.base_state;
        for input in self.input_buffer.iter() {
            self.entity.apply_input(level, &mut new_state, input, dt);
        }

        // Assign calculated state
        self.state = new_state;

        // Use the newly calculated state as the base
        if server {
            self.base_state = self.state;
            self.input_buffer.clear();
        }

        // Record entity states
        self.state_buffer.push_front((tick, new_state));
        if self.state_buffer.len() > self.state_buffer_size {
            self.state_buffer.pop_back();
        }

    }


    // Drawing ----------------------------------------------------------------
    pub fn draw(
        &mut self,
        renderer: &mut Renderer,
        rng: &mut XorShiftRng,
        level: &Level, dt: f32, u: f32
    ) {

        let state = if self.local() {
            level.interpolate_state(&self.state, &self.last_state, u)

        } else {
            // TODO have both server and client use the same offset here
            // for correct latency compensation
            let offset = self.offset_state_pair(3); // TODO configure
            level.interpolate_state(&offset.0, &offset.1, u)
        };

        self.drawable.draw(renderer, rng, level, state, dt, u);

    }


    // Serialization ----------------------------------------------------------
    pub fn serialize_state(&self, owner: &ConnectionID) -> Vec<u8> {

        let mut data = [
            (self.local_id >> 8) as u8,
            self.local_id as u8,
            self.entity.typ()

        ].to_vec();

        // Set local flag if we're serializing for the owner
        let mut state = self.state;
        if self.owned_by(owner) {
            state.flags |= 0x01;
        }
        data.extend(state.serialize());
        data

    }


    // Events -----------------------------------------------------------------
    pub fn server_created(&mut self, tick: u8) {
        self.entity.server_event_created(&self.state, tick);
    }

    pub fn client_created(&mut self, tick: u8) {
        self.entity.client_event_created(&self.state, tick);
        self.drawable.event_created(&self.state, tick);
    }

    pub fn server_destroyed(&mut self, tick: u8) {
        self.entity.server_event_destroyed(&self.state, tick);
    }

    pub fn client_destroyed(&mut self, tick: u8) {
        self.entity.client_event_destroyed(&self.state, tick);
        self.drawable.event_destroyed(&self.state, tick);
    }

}


// Helpers --------------------------------------------------------------------
pub fn tick_is_more_recent(a: u8, b: u8) -> bool {
    (a > b) && (a - b <= 128) || (b > a) && (b - a > 128)
}

