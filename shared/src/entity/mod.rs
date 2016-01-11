// Internal Dependencies ------------------------------------------------------
mod entity;
mod input;
mod state;
pub mod traits;


// Re-Exports -----------------------------------------------------------------
pub use entity::entity::Entity as Entity;
pub use entity::input::EntityInput as Input;
pub use entity::state::EntityState as State;


// Utilities ------------------------------------------------------------------
pub fn tick_is_more_recent(a: u8, b: u8) -> bool {
    (a > b) && (a - b <= 255 / 2) || (b > a) && (b - a > 255 / 2)
}

