// Internal Dependencies ------------------------------------------------------
use entity;


// Entity Registry Trait ------------------------------------------------------
pub trait EntityRegistry<S: entity::State> {
    fn entity_from_type_id(&self, type_id: u8) -> entity::Entity<S>;
}

