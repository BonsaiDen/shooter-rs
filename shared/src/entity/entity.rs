// External Dependencies ------------------------------------------------------
use allegro;
use allegro_primitives::PrimitivesAddon;
use rand::XorShiftRng;
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
use entity;
use arena::Arena;
use drawable::Drawable;
use particle::ParticleSystem;


// Top Level Entity Structure -------------------------------------------------
pub struct Entity {
    typ: Box<entity::Kind>,
    drawable: Box<Drawable>,
    owner: ConnectionID,
    is_alive: bool,
    local_id: u16
}

impl Entity {

    pub fn new(typ: Box<entity::Kind>, drawable: Box<Drawable>) -> Entity {
        Entity {
            typ: typ,
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
        self.typ.get_state()
    }

    pub fn set_state(&mut self, state: entity::State) {
        self.drawable.set_flags(state.flags);
        self.typ.set_state(state);
    }

    pub fn local(&self) -> bool {
        self.typ.is_local()
    }

    pub fn alive(&self) -> bool {
        self.is_alive
    }

    pub fn set_alive(&mut self, alive: bool) {
        self.is_alive = alive;
    }

    pub fn visible_to(&self, owner: &ConnectionID) -> bool {
        self.typ.visible_to(owner)
    }


    // Logic ------------------------------------------------------------------
    pub fn input(&mut self, input: entity::Input, max_inputs: usize) {
        self.typ.input(input, max_inputs);
    }

    pub fn tick_local(&mut self, arena: &Arena, dt: f32, temporary: bool) {
        self.typ.tick(arena, dt, temporary);
    }

    pub fn tick_remote(
        &mut self, arena: &Arena, dt: f32, remote_tick: u8, state: entity::State
    ) {
        self.typ.remote_tick(arena, dt, remote_tick, state);
    }


    // Drawing ----------------------------------------------------------------
    pub fn draw(
        &mut self,
        core: &allegro::Core, prim: &PrimitivesAddon,
        rng: &mut XorShiftRng, particle_system: &mut ParticleSystem,
        arena: &Arena, dt: f32, u: f32
    ) {
        self.drawable.draw(
            core, prim, rng,
            particle_system,
            arena,
            &*self.typ,
            dt, u
        );
    }


    // Serialization ----------------------------------------------------------
    pub fn serialize_inputs(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for input in self.typ.get_inputs().iter() {
            data.extend(input.serialize());
        }
        data
    }

    pub fn serialize_state(&self, owner: &ConnectionID) -> Vec<u8> {

        let mut data = [
            (self.local_id >> 8) as u8,
            self.local_id as u8,
            self.typ.kind_id()

        ].to_vec();

        // Set local flag if we're serializing for the owner
        let mut state = self.typ.get_state();
        if &self.owner == owner {
            state.flags |= 0x01;
        }
        data.extend(state.serialize());
        data

    }


    // Events -----------------------------------------------------------------
    pub fn create(&mut self) {
        self.typ.create();
    }

    pub fn destroy(&mut self) {
        self.typ.destroy();
        self.drawable.destroy();
    }

}

