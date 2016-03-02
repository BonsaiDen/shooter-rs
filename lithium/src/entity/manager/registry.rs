// Internal Dependencies ------------------------------------------------------
use level::Base as BaseLevel;
use entity::{Entity, State};
use renderer::Renderer;


// Entity Registry Trait ------------------------------------------------------
pub trait EntityRegistry<S: State, L: BaseLevel<S>, R: Renderer> {
    fn entity_from_type_id(&self, type_id: u8) -> Entity<S, L, R>;
}

