// External Dependencies ------------------------------------------------------
use lithium::{Level, Server, DefaultRenderer};


// Internal Dependencies ------------------------------------------------------
use shared::{Color, SharedEvent, SharedLevel, SharedState, SharedRegistry};
mod handler;


// Server Side Game Logic -----------------------------------------------------
pub struct Game {
    available_colors: Vec<Color>
}

impl Game {

    pub fn new() -> Game {
        Game {
            available_colors: Color::all_colored().into_iter().rev().collect()
        }
    }

    pub fn server(tick_rate: u32) -> Server<SharedEvent, SharedState, SharedLevel, DefaultRenderer>{
        Server::new(
            tick_rate, 1000, 75,
            Game::default_level(),
            Box::new(SharedRegistry),
            Box::new(Game::new())
        )
    }

    pub fn default_level() -> Level<SharedState, SharedLevel> {
        SharedLevel::create(384, 384, 16)
    }

}

