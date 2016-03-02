// External Dependencies ------------------------------------------------------
use lithium::Server;
use lithium::renderer::DefaultRenderer;
use lithium::level::Level as LithiumLevel;


// Internal Dependencies ------------------------------------------------------
use shared::entities;
use shared::color::Color;
use shared::event::Event;
use shared::level::Level;
use shared::state::State;
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

    pub fn server(tick_rate: u32) -> Server<Event, State, Level, DefaultRenderer>{
        Server::new(
            tick_rate, 1000, 75,
            Game::default_level(),
            Box::new(entities::Registry),
            Box::new(Game::new())
        )
    }

    pub fn default_level() -> LithiumLevel<State, Level> {
        Level::create(384, 384, 16)
    }

}

