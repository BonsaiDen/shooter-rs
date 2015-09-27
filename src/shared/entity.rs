use arena::Arena;

pub struct EntityInput {
    pub tick: u8,
    pub left: bool,
    pub right: bool,
    pub thrust: bool,
    pub fire: bool
}

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

pub trait Entity {

    fn is_local(&self) -> bool;

    fn kind_id(&self) -> u8;

    fn get_id(&self) -> u32;

    fn set_id(&mut self, id: u32);

    fn get_state(&mut self) -> EntityState;

    fn set_state(&mut self, state: EntityState);

    fn interpolate_state(&self, arena: &Arena, u: f32) -> EntityState;

    fn serialize_state(&self, buffer: &mut Vec<u8>);

    fn serialize_inputs(&self, buffer: &mut Vec<u8>);

    fn input(&mut self, input: EntityInput);

    fn tick(&mut self, arena: &Arena, dt: f32, set_last_state: bool);

    fn remote_tick(
        &mut self,
        arena: &Arena,
        dt: f32, set_last_state: bool, remote_tick: u8, state: EntityState
    );

}

