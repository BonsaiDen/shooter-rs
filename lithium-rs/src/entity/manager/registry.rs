// Internal Dependencies ------------------------------------------------------
use level::BaseLevel;
use renderer::Renderer;
use entity::{Entity, EntityState};


// Entity Registry Trait ------------------------------------------------------
pub trait EntityRegistry<S: EntityState, L: BaseLevel<S>, R: Renderer> {
    fn entity_from_type_id(&self, type_id: u8) -> Entity<S, L, R>;
}

