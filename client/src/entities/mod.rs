// External Dependencies ------------------------------------------------------
use lithium::entity;


// Internal Dependencies ------------------------------------------------------
use shared::state::State;

mod ship;
pub use entities::ship::Ship;


// Entity Registry ------------------------------------------------------------
pub struct Registry;
impl entity::Registry<State> for Registry {
    fn entity_from_type_id(&self, type_id: u8) -> entity::Entity<State> {
        match type_id {
            0 => ship::Ship::create_entity(1.0),
            _ => unreachable!()
        }
    }
}

