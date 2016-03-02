// External Dependencies ------------------------------------------------------
use std::net::SocketAddr;
use lithium::{Client, Level as LithiumLevel};


// Internal Dependencies ------------------------------------------------------
use entities;
use shared::level::Level;
use shared::event::Event;
use shared::state::State;
use renderer::AllegroRenderer;
mod handler;


// Game -----------------------------------------------------------------------
pub struct Game {
    state: GameState
}

impl Game {

    pub fn new() -> Game {
        Game {
            state: GameState::Disconnected
        }
    }

    pub fn client(server_addr: SocketAddr) -> Client<Event, State, Level, AllegroRenderer> {
        Client::new(
            server_addr,
            30,
            Game::default_level(),
            Box::new(entities::Registry)
        )
    }

    pub fn default_level() -> LithiumLevel<State, Level> {
        Level::create(384, 384, 16)
    }

}

// Game State -----------------------------------------------------------------
#[derive(PartialEq)]
enum GameState {
    Disconnected,
    Pending,
    Connected
}

