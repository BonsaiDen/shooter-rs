// External Dependencies ------------------------------------------------------
use std::net::SocketAddr;
use lithium::Client;


// Internal Dependencies ------------------------------------------------------
use entities;
use shared::event::Event;
use shared::level::Level;
use shared::state::State;
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

    pub fn client(server_addr: SocketAddr) -> Client<Event, Level, State> {
        Client::new(
            server_addr,
            Game::default_level(),
            Box::new(entities::Registry)
        )
    }

    pub fn default_level() -> Level {
        Level::new(384, 384, 16)
    }

}

// Game State -----------------------------------------------------------------
#[derive(PartialEq)]
enum GameState {
    Disconnected,
    Pending,
    Connected
}

