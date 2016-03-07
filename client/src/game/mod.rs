// External Dependencies ------------------------------------------------------
use std::net::SocketAddr;


// Internal Dependencies ------------------------------------------------------
use entities::Registry;
use shared::Lithium::{Client, Level};
use shared::{SharedEvent, SharedState, SharedLevel};
use renderer::AllegroRenderer;
mod handler;


// Game -----------------------------------------------------------------------
pub struct Game {
    state: GameState,
    last_connection_retry: f64
}

impl Game {

    pub fn new() -> Game {
        Game {
            state: GameState::Disconnected,
            last_connection_retry: 0.0
        }
    }

    pub fn client(server_addr: SocketAddr) -> Client<
        SharedEvent, SharedState, SharedLevel, AllegroRenderer
    > {
        Client::new(
            server_addr,
            30,
            Game::default_level(),
            Box::new(Registry)
        )
    }

    pub fn default_level() -> Level<SharedState, SharedLevel> {
        SharedLevel::create(384, 384, 16)
    }

}

// Game State -----------------------------------------------------------------
#[derive(PartialEq)]
enum GameState {
    Disconnected,
    Pending,
    Connected
}

