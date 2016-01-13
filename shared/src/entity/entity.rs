// External Dependencies ------------------------------------------------------
use rand::XorShiftRng;
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
use entity;
use arena::Arena;
use renderer::Renderer;
use entity::traits::{Base, Drawable};


// Top Level Entity Structure -------------------------------------------------
pub struct Entity {
    entity: Box<Base>,
    drawable: Box<Drawable>,
    owner: ConnectionID,
    is_alive: bool,
    local_id: u16
}

impl Entity {

    pub fn new(entity: Box<Base>, drawable: Box<Drawable>) -> Entity {
        Entity {
            entity: entity,
            drawable: drawable,
            owner: ConnectionID(0),
            is_alive: false,
            local_id: 0
        }
    }


    // Getter / Setter --------------------------------------------------------
    pub fn id(&self) -> u16 {
        self.local_id
    }

    pub fn set_id(&mut self, id: u16) {
        self.local_id = id;
    }

    pub fn owner(&self) -> &ConnectionID {
        &self.owner
    }

    pub fn set_owner(&mut self, owner: ConnectionID) {
        self.owner = owner;
    }

    pub fn owned_by(&mut self, owner: &ConnectionID) -> bool {
        self.owner == *owner
    }

    pub fn get_state(&self) -> entity::State {
        self.entity.get_state()
    }

    pub fn set_state(&mut self, state: entity::State) {
        self.drawable.flagged(state.flags);
        self.entity.set_state(state, true);
    }

    pub fn set_local_state(&mut self, state: entity::State) {
        self.entity.set_state(state, false);
    }

    pub fn set_remote_state(&mut self, tick: u8, state: entity::State) {
        self.entity.set_remote_state(tick, state);
    }

    pub fn local(&self) -> bool {
        self.entity.local()
    }

    pub fn alive(&self) -> bool {
        self.is_alive
    }

    pub fn set_alive(&mut self, alive: bool) {
        self.is_alive = alive;
    }

    pub fn visible_to(&self, owner: &ConnectionID) -> bool {
        self.entity.visible_to(owner)
    }


    // Logic ------------------------------------------------------------------
    pub fn local_input(&mut self, input: entity::Input, max_inputs: usize) -> Vec<u8> {

        self.entity.input(input, max_inputs);

        let mut inputs = Vec::new();
        for input in self.entity.pending_inputs().iter() {
            inputs.extend(input.serialize());
        }
        inputs

    }

    pub fn remote_input(&mut self, input: entity::Input, max_inputs: usize) {
        self.entity.input(input, max_inputs);
    }

    pub fn tick_client(&mut self, arena: &Arena, dt: f32) {
        self.entity.tick(arena, dt, false);
    }

    pub fn tick_server(&mut self, arena: &Arena, dt: f32) {
        self.entity.tick(arena, dt, true);
    }


    // Drawing ----------------------------------------------------------------
    pub fn draw(
        &mut self,
        renderer: &mut Renderer,
        rng: &mut XorShiftRng,
        arena: &Arena, dt: f32, u: f32
    ) {
        self.drawable.draw(renderer, rng, arena, &*self.entity, dt, u);
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
        let mut state = self.entity.get_state();
        if &self.owner == owner {
            state.flags |= 0x01;
        }
        data.extend(state.serialize());
        data

    }


    // Events -----------------------------------------------------------------
    pub fn created(&mut self) {
        self.entity.created();
    }

    pub fn destroyed(&mut self) {
        self.entity.destroyed();
        self.drawable.destroyed();
    }

}

