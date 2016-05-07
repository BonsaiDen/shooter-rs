#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

// Dependencies ---------------------------------------------------------------
extern crate num;
extern crate rand;
pub extern crate cobalt;
extern crate bincode;
extern crate rustc_serialize;


// Module Declarations --------------------------------------------------------
#[macro_use] mod timer;
mod client;
pub mod entity;
mod event;
mod idpool;
mod level;
pub mod network;
mod renderer;
mod server;


// Re-Exports -----------------------------------------------------------------
#[doc(inline)]
pub use cobalt as Cobalt;

#[doc(inline)]
pub use entity::*;

#[doc(inline)]
pub use level::*;

#[doc(inline)]
pub use event::Event;

#[doc(inline)]
pub use event::EventHandler;

#[doc(inline)]
pub use timer::TimerId;

#[doc(inline)]
pub use client::Client;

#[doc(inline)]
pub use client::Handle as ClientHandle;

#[doc(inline)]
pub use client::Handler as ClientHandler;

#[doc(inline)]
pub use client::Timer as ClientTimer;

#[doc(inline)]
pub use server::Server;

#[doc(inline)]
pub use server::Handle as ServerHandle;

#[doc(inline)]
pub use server::Handler as ServerHandler;

#[doc(inline)]
pub use server::Timer as ServerTimer;

#[doc(inline)]
pub use renderer::Renderer;

#[doc(inline)]
pub use renderer::DefaultRenderer;

