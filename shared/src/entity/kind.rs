// External Dependencies ------------------------------------------------------
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
use entity;
use arena::Arena;


// Entity Behavior Implementation Trait ---------------------------------------
pub trait EntityKind {

    // TODO cleanup API
    fn is_local(&self) -> bool;

    fn kind_id(&self) -> u8;

    fn get_state(&self) -> entity::State;

    fn set_state(&mut self, state: entity::State);

    fn get_inputs(&self) -> &Vec<entity::Input>;

    fn set_flags(&mut self, _: u8) {
    }

    fn interpolate_state(&self, arena: &Arena, u: f32) -> entity::State;

    fn visible_to(&self, _: &ConnectionID) -> bool {
        true
    }

    fn input(&mut self, input: entity::Input, max_inputs: usize);

    fn tick(&mut self, arena: &Arena, dt: f32, temporary: bool);

    fn remote_tick(
        &mut self,
        arena: &Arena,
        dt: f32, remote_tick: u8, state: entity::State
    );

    fn create(&mut self) {
    }

    fn destroy(&mut self) {
    }

}

