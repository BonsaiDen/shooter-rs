// Internal Dependencies ------------------------------------------------------
mod entity;
mod event;
mod input;
pub mod manager;
pub mod traits;


// Re-Exports -----------------------------------------------------------------
pub use self::entity::Entity as Entity;
pub use self::event::EntityEvent as Event;
pub use self::input::EntityInput as Input;
pub use self::traits::State as State;
pub use self::manager::EntityManager as Manager;
pub use self::manager::ControlState as ControlState;
pub use self::manager::config::EntityManagerConfig as ManagerConfig;
pub use self::manager::registry::EntityRegistry as Registry;

