// External Dependencies ------------------------------------------------------
use lithium::entity;
use lithium::renderer::DefaultRenderer;


// Internal Dependencies ------------------------------------------------------
use state::State;
use level::Level;

mod ship;
pub use entities::ship::Ship;


// Entity Registry ------------------------------------------------------------
pub struct Registry;
impl entity::Registry<State, Level, DefaultRenderer> for Registry {
    fn entity_from_type_id(&self, type_id: u8) -> entity::Entity<State, Level, DefaultRenderer> {
        match type_id {
            0 => ship::Ship::create_entity(1.0),
            _ => unreachable!()
        }
    }
}


// Noop Drawable --------------------------------------------------------------
pub struct DefaultDrawable;
impl entity::traits::Drawable<State, Level, DefaultRenderer> for DefaultDrawable {}

