use std::f32;

use arena::Arena;
use drawable::Drawable;
use cobalt::ConnectionID;
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};


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

    pub fn owner(&self) -> &ConnectionID {
        &self.owner
    }

    pub fn get_state(&self) -> EntityState {
        self.typ.get_state()
    }

    pub fn set_state(&mut self, state: EntityState) {
        self.drawable.set_flags(state.flags);
        self.typ.set_state(state);
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

    pub fn visible_to(&self, owner: &ConnectionID) -> bool {
        self.typ.visible_to(owner)
    }

    pub fn serialize(&self, owner: &ConnectionID) -> Vec<u8> {

        let mut data = [
            (self.local_id >> 8) as u8,
            self.local_id as u8,
            self.typ.kind_id()

        ].to_vec();

        // TODO check if owner is owner of entity and set local flag
        let mut state = self.typ.get_state();
        if &self.owner == owner {
            state.flags |= 0x01;
        }
        data.extend(state.serialize());
        data

    }

    pub fn inputs(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for input in self.typ.get_inputs().iter() {
            data.extend(input.serialize());
        }
        data
    }

    pub fn create(&mut self) {
        self.typ.create();
    }

    pub fn destroy(&mut self) {
        self.typ.destroy();
    }

}


// Entity Input Data ----------------------------------------------------------
#[derive(Debug, Copy, Clone, RustcEncodable, RustcDecodable)]
pub struct EntityInput {
    pub tick: u8,
    pub left: bool,
    pub right: bool,
    pub thrust: bool,
    pub fire: bool
}

impl EntityInput {

    pub fn encoded_size() -> usize {
        5
    }

    pub fn from_serialized(data: &[u8]) -> EntityInput {
        let state: EntityInput = decode(data).unwrap();
        state
    }

    pub fn serialize(&self) -> Vec<u8> {
        encode(&self, SizeLimit::Infinite).unwrap()
    }

}


// Entity State Data ----------------------------------------------------------
#[derive(Debug, Copy, Clone, RustcEncodable, RustcDecodable)]
pub struct EntityState {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub mx: f32,
    pub my: f32,
    pub flags: u8
}

impl EntityState {

    pub fn encoded_size() -> usize {
        21
    }

    pub fn from_serialized(data: &[u8]) -> EntityState {
        let state: EntityState = decode(data).unwrap();
        state
    }

    pub fn serialize(&self) -> Vec<u8> {
        encode(&self, SizeLimit::Infinite).unwrap()
    }

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

    fn get_state(&self) -> EntityState;

    fn set_state(&mut self, state: EntityState);

    fn get_inputs(&self) -> &Vec<EntityInput>;

    fn interpolate_state(&self, arena: &Arena, u: f32) -> EntityState;

    fn visible_to(&self, owner: &ConnectionID) -> bool {
        true
    }

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


// Utilities ------------------------------------------------------------------
pub fn tick_is_more_recent(a: u8, b: u8) -> bool {
    (a > b) && (a - b <= 255 / 2) || (b > a) && (b - a > 255 / 2)
}

pub fn apply_input_to_state(
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



