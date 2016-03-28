// External Dependencies ------------------------------------------------------
use shared::Lithium::{Level, Server, DefaultRenderer, Entity, ServerHandle as Handle};


// Internal Dependencies ------------------------------------------------------
use shared::{Color, SharedEvent, SharedLevel, SharedState, SharedRegistry};
mod handler;


// Type Aliases ---------------------------------------------------------------
pub type ServerHandle<'a> = Handle<'a, Game, DefaultRenderer, SharedRegistry, SharedLevel, SharedEvent, SharedState>;
pub type ServerLevel = Level<SharedState, SharedLevel>;
pub type ServerEntity = Entity<SharedState, SharedLevel, DefaultRenderer>;


// Server Side Game Logic -----------------------------------------------------
pub struct Game {
    available_colors: Vec<Color>,
    loopback_mode: bool,
    counter: u32
}

impl Game {

    pub fn new(loopback_mode: bool) -> Game {
        Game {
            available_colors: Color::all_colored().into_iter().rev().collect(),
            loopback_mode: loopback_mode,
            counter: 1
        }
    }

    pub fn server(
        tick_rate: u32,
        loopback_mode: bool

    ) -> Server<
        Game, DefaultRenderer,
        SharedRegistry, SharedLevel, SharedEvent, SharedState
    > {
        Server::new(
            tick_rate, 1000, 75,
            Game::default_level(),
            SharedRegistry,
            Game::new(loopback_mode)
        )
    }

    pub fn default_level() -> Level<SharedState, SharedLevel> {
        SharedLevel::create(384, 384, 16)
    }

    fn count(&mut self, handle: ServerHandle) {
        handle.timer.schedule(Box::new(|game, handle| {
            println!("[Server] Counter: {}", game.counter);
            game.counter += 1;
            game.count(handle);

        }), 1000);
    }

}

