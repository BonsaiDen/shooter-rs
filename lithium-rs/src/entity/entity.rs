// External Dependencies ------------------------------------------------------
use cobalt::ConnectionID;
use std::collections::VecDeque;


// Internal Dependencies ------------------------------------------------------
use renderer::Renderer;
use level::{Level, BaseLevel};
use entity::{EntityEvent, EntityInput, EntityState};
use entity::traits::{BaseEntity, DrawableEntity};


// Entity Wrapper Structure ---------------------------------------------------
pub struct Entity<S: EntityState, L: BaseLevel<S>, R: Renderer> {

    entity: Box<BaseEntity<S, L>>,
    drawable: Box<DrawableEntity<S, L, R>>,
    owner: Option<ConnectionID>,
    is_alive: bool,
    is_visible: bool,
    local_id: u16,

    // State
    state: S,
    base_state: S,
    last_state: S,
    confirmed_state: Option<(u8, S)>,
    state_buffer: VecDeque<(u8, S)>,

    // Inputs
    input_buffer: VecDeque<EntityInput>,
    confirmed_input_tick: u8,
    initial_input: bool,
    serialized_inputs: Option<Vec<u8>>,

    // Configuration
    input_buffer_size: usize,
    state_buffer_size: usize

}

impl<S: EntityState, L: BaseLevel<S>, R: Renderer> Entity<S, L, R> {

    pub fn new(
        entity: Box<BaseEntity<S, L>>,
        drawable: Box<DrawableEntity<S, L, R>>

    ) -> Entity<S, L, R> {
        Entity {

            // Entity Behavior
            entity: entity,

            // Entity Rendering
            drawable: drawable,

            // Owner of the Entity
            owner: None,

            // Whether the entity is still alive or should be destroyed
            is_alive: false,

            // Whether the entity is currently visible and receicving state
            // updates (client only)
            is_visible: false,

            // Locally used Entity ID
            local_id: 0,

            // Current - calculated - entity state
            state: S::default(),

            // Current base state (before apply pending inputs)
            base_state: S::default(),

            // Previously caluclated state for interpolation purposes
            last_state: S::default(),

            // Last confirmed remote state (client only)
            confirmed_state: None,

            // List of previous entity states for client-side interpolation
            // and server-side latency compensation
            state_buffer: VecDeque::new(),

            // Pending inputs (client only)
            input_buffer: VecDeque::new(),

            // Serialized inputs (client only)
            serialized_inputs: None,

            // Last tick for which input was received (server only)
            confirmed_input_tick: 0,
            initial_input: true,

            // Configuration
            input_buffer_size: 30,
            state_buffer_size: 30

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
        self.state.flags() & 0x01 == 0x01
    }

    pub fn alive(&self) -> bool {
        self.is_alive
    }

    pub fn set_alive(&mut self, alive: bool) {
        self.is_alive = alive;
    }

    pub fn set_buffer_size(&mut self, ticks: usize) {
        self.input_buffer_size = ticks;
        self.state_buffer_size = ticks;
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


    // Visibility -------------------------------------------------------------
    pub fn visible_to(&self, owner: &ConnectionID) -> bool {
        self.entity.visible_to(owner)
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn show(&mut self, tick: u8) {
        self.is_visible = true;
        self.event(EntityEvent::Show(tick));
    }

    pub fn hide(&mut self, tick: u8) {
        self.is_visible = false;
        self.event(EntityEvent::Hide(tick));
    }


    // State ------------------------------------------------------------------
    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn buffered_state(&self, tick_offset: usize) -> &S {
        let buffer_len = self.state_buffer.len();
        if buffer_len > 0 && tick_offset < buffer_len {
            &self.state_buffer[tick_offset].1

        } else {
            &self.state
        }
    }

    fn buffered_states(
        &self, tick_offset: usize

    ) -> (&S, &S) {
        let buffer_len = self.state_buffer.len();
        if buffer_len > 0 && tick_offset + 1 < buffer_len {
            (
                &self.state_buffer[tick_offset].1,
                &self.state_buffer[tick_offset + 1].1
            )
        } else {
            (
                &self.state,
                &self.last_state
            )
        }
    }

    /*
    pub fn rewind_state(&mut self, tick_offset: usize) {
        // TODO clean up
        let (state, last_state) = {
            let states = self.buffered_states(tick_offset);
            (states.0.clone(), states.1.clone())
        };
        self.state.set_to(&state);
        self.base_state.set_to(&state);
        self.last_state.set_to(&last_state);
    }

    pub fn forward_state(&mut self) {
        self.rewind_state(0);
    }
    */
    pub fn set_state(&mut self, state: S) {
        self.set_entity_state(state, true);
    }

    pub fn set_remote_state(&mut self, state: S) {
        self.set_entity_state(state, false);
    }

    pub fn set_confirmed_state(&mut self, tick: u8, state: S) {
        self.confirmed_state = Some((tick, state));
    }

    fn set_entity_state(&mut self, new_state: S, override_last: bool) {

        let old_flags = self.state.flags();
        if override_last {
            self.last_state.set_to(&new_state);

        } else {
            self.last_state.set_to(&self.state);
        };

        self.base_state.set_to(&new_state);
        self.state.set_to(&new_state);

        if old_flags != new_state.flags() {
            self.event(EntityEvent::Flags(new_state.flags()));
        }

    }


    // Input ------------------------------------------------------------------
    pub fn local_input(&mut self, input: EntityInput) {

        self.input(input);

        let mut serialized_inputs = Vec::new();
        for input in &self.input_buffer {
            serialized_inputs.extend(input.serialize());
        }

        self.serialized_inputs = Some(serialized_inputs);

    }

    pub fn serialized_inputs(&mut self) -> Option<Vec<u8>> {
        if let Some(inputs) = self.serialized_inputs.take() {
            Some(inputs)

        } else {
            None
        }
    }

    pub fn remote_input(&mut self, input: EntityInput) {

        if self.initial_input ||  tick_is_more_recent(
            input.tick,
            self.confirmed_input_tick
        ) {
            self.initial_input = false;
            self.confirmed_input_tick = input.tick;
            self.input(input);
        }

    }

    fn input(&mut self, input: EntityInput) {

        self.input_buffer.push_back(input);

        // Drop outdated inputs
        if self.input_buffer.len() > self.input_buffer_size {
            self.input_buffer.pop_front();
        }

    }


    // Ticking ----------------------------------------------------------------
    pub fn client_tick(&mut self, level: &Level<S, L>, tick: u8, dt: f32) {
        self.event(EntityEvent::Tick(tick, dt)); // TODO useful?
        self.tick(level, tick, dt, false);
    }

    pub fn server_tick(&mut self, level: &Level<S, L>, tick: u8, dt: f32) {
        self.event(EntityEvent::Tick(tick, dt)); // TODO useful?
        self.tick(level, tick, dt, true);
    }

    pub fn tick(&mut self, level: &Level<S, L>, tick: u8, dt: f32, server: bool) {

        // Check if we have a remote state
        if let Some((confirmed_tick, confirmed_state)) = self.confirmed_state.take() {

            // Set the current state as the last state and tkae over the
            // confirmed state as new base state
            self.set_entity_state(confirmed_state, false);

            // Drop all inputs confirmed by the remote so the remaining ones
            // get applied on top of the new base state
            self.input_buffer.retain(|input| {
                tick_is_more_recent(input.tick, confirmed_tick)
            });

        // Otherwise reset the local state and re-apply the inputs on top of it
        } else {
            self.last_state.set_to(&self.state);
            self.state.set_to(&self.base_state);
        }

        // Apply unconfirmed inputs on top of last state confirmed by the server
        let mut new_state = self.base_state.clone();
        for input in &self.input_buffer {
            self.entity.apply_input(level, &mut new_state, input, dt);
        }

        // Assign calculated state
        self.state.set_to(&new_state);

        // Use the newly calculated state as the base
        if server {
            self.base_state.set_to(&self.state);
            self.input_buffer.clear();
        }

        // Record entity states
        self.state_buffer.push_front((tick, new_state));
        if self.state_buffer.len() > self.state_buffer_size {
            self.state_buffer.pop_back();
        }

    }


    // Drawing ----------------------------------------------------------------
    pub fn draw(&mut self, renderer: &mut R, level: &Level<S, L>) {

        let state = if self.local() {
            level.interpolate_entity_state(renderer, &self.state, &self.last_state)

        } else {
            let offset = self.buffered_states(renderer.interpolation_ticks());
            level.interpolate_entity_state(renderer, &offset.0, &offset.1)
        };

        self.drawable.draw(renderer, level, state);

    }


    // Serialization ----------------------------------------------------------
    pub fn header_size() -> usize {
        5
    }

    pub fn serialize_state(&self, owner: &ConnectionID) -> Vec<u8> {

        // Entity Header
        let is_visible = self.visible_to(owner);
        let mut data = [
            (self.local_id >> 8) as u8, self.local_id as u8,
            self.entity.type_id(),
            self.confirmed_input_tick,
            is_visible as u8

        ].to_vec();

        // Serialize state only for visible entities
        if is_visible {

            // Create a copy of the current state
            let mut state: S = self.state.clone();

            // Set local flag if we're serializing for the owner
            if self.owned_by(owner) {
                let flags = state.flags();
                state.set_flags(flags | 0x01);
            }

            // Invoke type specific serialization handler
            self.entity.serialize_state(&mut state, owner);

            data.extend(state.serialize());
        }

        data

    }


    // Events -----------------------------------------------------------------
    pub fn event(&mut self, event: EntityEvent) {
        self.entity.event(&event, &self.state);
        self.drawable.event(&event, &self.state);
    }

}


// Helpers --------------------------------------------------------------------
pub fn tick_is_more_recent(a: u8, b: u8) -> bool {
    (a > b) && (a - b <= 128) || (b > a) && (b - a > 128)
}

