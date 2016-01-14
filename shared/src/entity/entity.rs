// External Dependencies ------------------------------------------------------
use rand::XorShiftRng;
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
use entity;
use level::Level;
use renderer::Renderer;
use entity::traits::{Base, Drawable};


// Top Level Entity Structure -------------------------------------------------
pub struct Entity {
    entity: Box<Base>,
    drawable: Box<Drawable>,
    owner: ConnectionID,
    is_alive: bool,
    local_id: u16,
    state: entity::State,
    base_state: entity::State,
    last_state: entity::State,
    remote_state: Option<(u8, entity::State)>,
    input_states: Vec<entity::Input>,
    last_input_tick: u8,
}

impl Entity {

    pub fn new(entity: Box<Base>, drawable: Box<Drawable>) -> Entity {
        Entity {

            // Entity Behavior
            entity: entity,

            // Entity Rendering
            drawable: drawable,

            // Owner of the Entity
            owner: ConnectionID(0), // TODO make this an option?

            // Whether the entity is still alive or should be destroyed
            is_alive: false,

            // Local Entity ID
            local_id: 0,

            // Current - calculated - entity state
            state: entity::State::default(),

            // Current base state (before apply pending inputs)
            base_state: entity::State::default(),

            // Previously caluclated state for interpolation purposes
            last_state: entity::State::default(),

            // Last confirmed remote state (client only)
            remote_state: None,

            // Pending inputs (client only)
            input_states: Vec::new(),

            // Last tick for which input was received (server only)
            last_input_tick: 0

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
    pub fn owner(&self) -> &ConnectionID {
        &self.owner
    }

    pub fn set_owner(&mut self, owner: ConnectionID) {
        self.owner = owner;
    }

    pub fn owned_by(&mut self, owner: &ConnectionID) -> bool {
        self.owner == *owner
    }


    pub fn visible_to(&self, owner: &ConnectionID) -> bool {
        self.entity.visible_to(owner)
    }


    // State ------------------------------------------------------------------
    pub fn get_state(&self) -> entity::State {
        self.state
    }

    pub fn set_state(&mut self, state: entity::State) {
        self.set_entity_state(state, true);
    }

    pub fn set_local_state(&mut self, state: entity::State) {
        self.set_entity_state(state, false);
    }

    pub fn set_remote_state(&mut self, tick: u8, state: entity::State) {
        self.remote_state = Some((tick, state));
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
    pub fn local_input(&mut self, input: entity::Input, max_inputs: usize) -> Vec<u8> {

        self.input(input, max_inputs);

        let mut inputs = Vec::new();
        for input in self.input_states.iter() {
            inputs.extend(input.serialize());
        }
        inputs

    }

    pub fn remote_input(&mut self, input: entity::Input, max_inputs: usize) {
        self.input(input, max_inputs);
    }

    fn input(&mut self, input: entity::Input, max_inputs: usize) {

        // Ignore inputs for past ticks
        if self.input_states.len() == 0 || tick_is_more_recent(
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


    // Ticking ----------------------------------------------------------------
    pub fn client_tick(&mut self, level: &Level, tick: u8, dt: f32) {
        self.entity.client_event_tick(level, &self.state, tick, dt);
        self.tick(level, dt, false);
    }

    pub fn server_tick(&mut self,  level: &Level, tick: u8, dt: f32) {
        self.entity.server_event_tick(level, &self.state, tick, dt);
        self.tick(level, dt, true);
    }

    fn tick(&mut self, level: &Level, dt: f32, server: bool) {

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
                tick_is_more_recent(input.tick, remote_tick)
            });

        // Otherwise reset the local state and re-apply the inputs on top of it
        } else {
            self.last_state = self.state;
            self.state = self.base_state;
        }

        // Apply unconfirmed inputs on top of last state confirmed by the server
        self.state = self.entity.apply_inputs(
            self.base_state, &self.input_states, level, dt
        );

        // Use the newly calculated state as the base
        if server {
            self.base_state = self.state;
            self.input_states.clear();
        }

    }


    // Drawing ----------------------------------------------------------------
    pub fn draw(
        &mut self,
        renderer: &mut Renderer,
        rng: &mut XorShiftRng,
        level: &Level, dt: f32, u: f32
    ) {
        let state = level.interpolate_state(&self.state, &self.last_state, u);
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
        // TODO clean up?
        let mut state = self.state;
        if &self.owner == owner {
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
    (a > b) && (a - b <= 255 / 2) || (b > a) && (b - a > 255 / 2)
}

