use cobalt::ConnectionID;
use arena::Arena;
use drawable::Drawable;

// Top Level Entity Structure -------------------------------------------------
pub struct Entity {
    pub typ: Box<EntityType>,
    pub drawable: Box<Drawable>,
    owner: ConnectionID,
    is_alive: bool,
    local_id: u16
}

impl Entity {

    pub fn new(typ: Box<EntityType>, drawable: Box<Drawable>) -> Entity {
        Entity {
            typ: typ,
            drawable: drawable,
            owner: ConnectionID(0),
            is_alive: false,
            local_id: 0
        }
    }

    pub fn id(&self) -> u16 {
        self.local_id
    }

    pub fn set_id(&mut self, id: u16) {
        self.local_id = id;
    }

    pub fn alive(&self) -> bool {
        self.is_alive
    }

    pub fn set_alive(&mut self, alive: bool) {
        self.is_alive = alive;
    }

    pub fn set_owner(&mut self, owner: ConnectionID) {
        self.owner = owner;
    }

    pub fn owned_by(&mut self, owner: &ConnectionID) -> bool {
        self.owner == *owner
    }

}


// Entity Input Data ----------------------------------------------------------
pub struct EntityInput {
    pub tick: u8,
    pub left: bool,
    pub right: bool,
    pub thrust: bool,
    pub fire: bool
}


// Entity State Data ----------------------------------------------------------
#[derive(Copy, Clone)]
pub struct EntityState {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub mx: f32,
    pub my: f32,
    pub flags: u8
}

impl Default for EntityState {
    fn default() -> EntityState {
        EntityState {
            x: 0.0,
            y: 0.0,
            r: 0.0,
            mx: 0.0,
            my: 0.0,
            flags: 0
        }
    }
}


// Underlying Entity Type Trait -----------------------------------------------
pub trait EntityType {

    fn is_local(&self) -> bool;

    fn kind_id(&self) -> u8;

    fn get_state(&mut self) -> EntityState;

    fn set_state(&mut self, state: EntityState);

    fn interpolate_state(&self, arena: &Arena, u: f32) -> EntityState;

    fn serialize_state(&self, buffer: &mut Vec<u8>);

    fn serialize_inputs(&self, buffer: &mut Vec<u8>);

    fn create(&mut self) {
    }

    fn set_flags(&mut self, _: u8) {
    }

    fn destroy(&mut self) {
    }

    fn input(&mut self, input: EntityInput);

    fn tick(&mut self, arena: &Arena, dt: f32);

    fn remote_tick(
        &mut self,
        arena: &Arena,
        dt: f32, remote_tick: u8, state: EntityState
    );

}

