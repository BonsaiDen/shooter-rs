// External Dependencies ------------------------------------------------------
use std::net::SocketAddr;


// Internal Dependencies ------------------------------------------------------
use entities::Registry;
use shared::Lithium::{Client, Level, Renderer};
use shared::{SharedEvent, SharedState, SharedLevel};
use renderer::AllegroRenderer;
mod handler;

use self::handler::Handle;


// Game -----------------------------------------------------------------------
pub struct Game {
    state: GameState,
    last_connection_retry: f64,
    server_addr: SocketAddr
}

impl Game {

    pub fn new(server_addr: SocketAddr) -> Game {
        Game {
            state: GameState::Disconnected,
            last_connection_retry: 0.0,
            server_addr: server_addr
        }
    }

    pub fn client() -> Client<
        SharedEvent, SharedState, SharedLevel, AllegroRenderer
    > {
        Client::new(
            30,
            Game::default_level(),
            Box::new(Registry)
        )
    }

    pub fn default_level() -> Level<SharedState, SharedLevel> {
        SharedLevel::create(384, 384, 16)
    }

    fn reset(&mut self, client: Handle) {
        client.renderer.set_fps(60);
        client.renderer.set_title("Rustgame: Shooter");
        client.renderer.resize(client.level.width() as i32, client.level.height() as i32);
    }

}

// Game State -----------------------------------------------------------------
#[derive(PartialEq)]
enum GameState {
    Disconnected,
    Pending,
    Connected
}

