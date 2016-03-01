// Internal Dependencies ------------------------------------------------------
use entity::{Entity, State};


// Entity Registry Trait ------------------------------------------------------
pub trait EntityRegistry<S: State> {
    fn entity_from_type_id(&self, type_id: u8) -> Entity<S>;
}

