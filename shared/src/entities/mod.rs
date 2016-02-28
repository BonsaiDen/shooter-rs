// External Dependencies ------------------------------------------------------
use lithium::entity;


// Internal Dependencies ------------------------------------------------------
use state;
mod ship;
pub use entities::ship::Ship;


// Entity Registry ------------------------------------------------------------
pub struct Registry;
impl entity::Registry<state::State> for Registry {
    fn entity_from_type_id(&self, type_id: u8) -> entity::Entity<state::State> {
        match type_id {
            0 => ship::Ship::create_entity(1.0),
            _ => unreachable!()
        }
    }
}

