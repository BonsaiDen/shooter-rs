// External Dependencies ------------------------------------------------------
use lithium::{Entity, EntityRegistry};


// Internal Dependencies ------------------------------------------------------
use shared::{SharedLevel, SharedState};
use renderer::AllegroRenderer;

mod ship;
pub use entities::ship::RenderedShip;


// Entity Registry ------------------------------------------------------------
pub struct Registry;
impl EntityRegistry<SharedState, SharedLevel, AllegroRenderer> for Registry {
    fn entity_from_type_id(&self, type_id: u8) -> Entity<SharedState, SharedLevel, AllegroRenderer> {
        match type_id {
            0 => RenderedShip::create_entity(1.0),
            _ => unreachable!()
        }
    }
}

