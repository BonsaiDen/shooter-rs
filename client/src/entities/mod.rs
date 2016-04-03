// Internal Dependencies ------------------------------------------------------
use shared::Lithium::{Entity, EntityRegistry};
use shared::{SharedLevel, SharedState};
use renderer::Renderer;

mod ship;
pub use entities::ship::RenderedShip;


// Entity Registry ------------------------------------------------------------
pub struct Registry;
impl EntityRegistry<SharedState, SharedLevel, Renderer> for Registry {
    fn entity_from_type_id(&self, type_id: u8) -> Entity<SharedState, SharedLevel, Renderer> {
        match type_id {
            0 => RenderedShip::create_entity(1.0),
            _ => unreachable!()
        }
    }
}

