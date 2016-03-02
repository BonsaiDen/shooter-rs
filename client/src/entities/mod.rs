// External Dependencies ------------------------------------------------------
use lithium::entity;


// Internal Dependencies ------------------------------------------------------
use shared::level::Level;
use shared::state::State;
use renderer::AllegroRenderer;

mod ship;
pub use entities::ship::Ship;


// Entity Registry ------------------------------------------------------------
pub struct Registry;
impl entity::Registry<State, Level, AllegroRenderer> for Registry {
    fn entity_from_type_id(&self, type_id: u8) -> entity::Entity<State, Level, AllegroRenderer> {
        match type_id {
            0 => ship::Ship::create_entity(1.0),
            _ => unreachable!()
        }
    }
}

