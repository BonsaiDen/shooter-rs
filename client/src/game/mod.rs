// External Dependencies ------------------------------------------------------
use std::net::SocketAddr;


// Internal Dependencies ------------------------------------------------------
mod handler;
mod views;
use entities::Registry;
use shared::Lithium::{Client, ClientHandle as Handle, Entity, Level, Renderer};
use shared::{SharedEvent, SharedState, SharedLevel};
use renderer::AllegroRenderer;
use self::views::View;


// Type Aliases ---------------------------------------------------------------
pub type ClientHandle<'a> = Handle<'a, SharedEvent, SharedState, SharedLevel, AllegroRenderer, Game>;
pub type ClientEntity = Entity<SharedState, SharedLevel, AllegroRenderer>;
pub type ClientLevel = Level<SharedState, SharedLevel>;


// Game -----------------------------------------------------------------------
pub struct Game {
    server_addr: Option<SocketAddr>,
    view: Option<Box<View>>,
    next_view: Option<Box<View>>
}

impl Game {

    pub fn new(server_addr: Option<SocketAddr>) -> Game {
        Game {
            server_addr: server_addr,
            view: Some(Box::new(views::InitView)),
            next_view: None
        }
    }

    pub fn client(server_addr: Option<SocketAddr>) -> Client<
        SharedEvent, SharedState, SharedLevel, AllegroRenderer, Game
    > {
        Client::new(
            30,
            Game::default_level(),
            Box::new(Registry),
            Game::new(server_addr)
        )
    }

    pub fn default_level() -> Level<SharedState, SharedLevel> {
        SharedLevel::create(384, 384, 16)
    }

    pub fn set_view(&mut self, view: Box<View>) {
        self.next_view = Some(view);
    }

    fn reset(&mut self, client: &mut ClientHandle) {
        client.renderer.set_fps(60);
        client.renderer.set_title("Rustgame: Shooter");
        client.renderer.resize(client.level.width() as i32, client.level.height() as i32);
    }

}

