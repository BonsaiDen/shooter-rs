// Internal Dependencies ------------------------------------------------------
mod control;
mod entity;
mod event;
mod input;
pub mod manager;
mod state;
pub mod traits;


// Re-Exports -----------------------------------------------------------------
pub use self::entity::Entity as Entity;
pub use self::event::EntityEvent as Event;
pub use self::control::EntityControl as ControlState;
pub use self::input::EntityInput as Input;
pub use self::state::EntityState as State;
pub use self::manager::EntityManager as Manager;
pub use self::manager::config::EntityManagerConfig as ManagerConfig;
pub use self::manager::registry::EntityRegistry as Registry;

