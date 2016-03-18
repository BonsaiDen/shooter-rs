#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

// Dependencies ---------------------------------------------------------------
extern crate num;
extern crate rand;
extern crate cobalt;
extern crate bincode;
extern crate rustc_serialize;


// Module Declarations --------------------------------------------------------
mod client;
pub mod entity;
mod event;
mod level;
pub mod network;
mod renderer;
mod server;
mod idpool;


// Re-Exports -----------------------------------------------------------------
pub use cobalt as Cobalt;
pub use entity::*;
pub use level::*;
pub use event::Event;
pub use event::EventHandler;
pub use idpool::IdPool;
pub use client::Client;
pub use client::Handle as ClientHandle;
pub use client::Handler as ClientHandler;
pub use server::Server;
pub use server::Handle as ServerHandle;
pub use server::Handler as ServerHandler;
pub use renderer::Renderer;
pub use renderer::DefaultRenderer;

