// External Dependencies ------------------------------------------------------
use lithium::{Entity, EntityRegistry, DrawableEntity, DefaultRenderer};


// Internal Dependencies ------------------------------------------------------
use state::SharedState;
use level::SharedLevel;

mod ship;
pub use entities::ship::Ship;


// Entity Registry ------------------------------------------------------------
pub struct Registry;
impl EntityRegistry<SharedState, SharedLevel, DefaultRenderer> for Registry {
    fn entity_from_type_id(&self, type_id: u8) -> Entity<SharedState, SharedLevel, DefaultRenderer> {
        match type_id {
            0 => ship::Ship::create_entity(1.0),
            _ => unreachable!()
        }
    }
}


// Noop Drawable --------------------------------------------------------------
pub struct DefaultDrawable;
impl DrawableEntity<SharedState, SharedLevel, DefaultRenderer> for DefaultDrawable {}

