use arena::Arena;
use drawable::ZeroDrawable;
use entity::{
    EntityType,
    EntityInput,
    EntityState,
    Entity,
    tick_is_more_recent,
    apply_input_to_state
};

pub struct Ship {
    state: EntityState,
    base_state: EntityState,
    last_state: EntityState,
    max_speed: f32,
    acceleration: f32,
    rotation: f32,
    input_states: Vec<EntityInput>,
    last_input_tick: u8
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
            max_speed: 90.0 * scale,
            acceleration: 2.0 * scale,
            rotation: 120.0,
            input_states: Vec::new(),
            last_input_tick: 0
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

impl EntityType for Ship {

    fn is_local(&self) -> bool {
        self.state.flags & 0x01 == 0x01
    }

    fn kind_id(&self) -> u8 {
        0
    }

    fn get_state(&self) -> EntityState  {
        self.state
    }

    fn set_state(&mut self, state: EntityState) {
        self.state = state;
        self.set_flags(state.flags);
        self.last_state = state;
        self.base_state = state;
    }

    fn get_inputs(&self) -> &Vec<EntityInput> {
        &self.input_states
    }

    fn interpolate_state(&self, arena: &Arena, u: f32) -> EntityState {
        arena.interpolate_state(&self.state, &self.last_state, u)
    }

    fn input(&mut self, input: EntityInput, max_inputs: usize) {

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

    fn remote_tick(
        &mut self,
        arena: &Arena,
        dt: f32, remote_tick: u8, state: EntityState
    ) {
        self.apply_remote_state(remote_tick, state);
        self.apply_inputs(arena, dt);
    }

    fn tick(&mut self, arena: &Arena, dt: f32, temporary: bool) {

        self.apply_local_state();
        self.apply_inputs(arena, dt);

        // Set the tick state as the new base state and clear pending inputs
        if temporary == false {
            self.base_state = self.state;
            self.input_states.clear();
        }

    }

}

