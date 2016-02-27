// External Dependencies ------------------------------------------------------
use std::net::SocketAddr;
use lithium::Client;


// Internal Dependencies ------------------------------------------------------
use entities;
use shared::event::Event;
use shared::level::Level;
mod runnable;


// Game -----------------------------------------------------------------------
pub struct Game {
    state: State
}

impl Game {

    pub fn new() -> Game {
        Game {
            state: State::Disconnected
        }
    }

    pub fn client(server_addr: SocketAddr) -> Client<Event, Level> {
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
enum State {
    Disconnected,
    Pending,
    Connected
}

