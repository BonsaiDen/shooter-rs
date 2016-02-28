// External Dependencies ------------------------------------------------------
use lithium::Server;


// Internal Dependencies ------------------------------------------------------
use shared::entities;
use shared::event::Event;
use shared::level::Level;
use shared::color::Color;
mod handler;


// Server Side Game Logic -----------------------------------------------------
pub struct Game {
    available_colors: Vec<Color>
}

impl Game {

    pub fn new() -> Game {
        Game {
            available_colors: Color::all_colored().into_iter().rev().collect(),
        }
    }

    pub fn server(tick_rate: u32) -> Server<Event, Level> {
        Server::new(
            tick_rate, 1000, 75,
            Game::default_level(),
            Box::new(entities::Registry),
            Box::new(Game::new())
        )
    }

    pub fn default_level() -> Level {
        Level::new(384, 384, 16)
    }

}

