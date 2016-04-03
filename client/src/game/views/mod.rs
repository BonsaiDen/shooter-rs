// Internal Dependencies ------------------------------------------------------
use renderer::Renderer;
use shared::SharedEvent;
use shared::Lithium::Cobalt::ConnectionID;
use super::{Game, ClientHandle, ClientLevel, ClientEntity};


// Views ----------------------------------------------------------------------
mod connect;
mod init;
mod menu;
mod game;
pub use self::connect::ConnectView;
pub use self::init::InitView;
pub use self::menu::MenuView;
pub use self::game::GameView;


// View Trait -----------------------------------------------------------------
pub trait View {

    fn name(&self) -> &str;

    fn push(&mut self, _: &mut Game, _: &mut ClientHandle) {}

    fn init(&mut self, _: &mut Game, _: &mut ClientHandle) {}

    fn connect(&mut self, _: &mut Game, _: &mut ClientHandle) {}

    fn disconnect(&mut self, _: &mut Game, _: &mut ClientHandle, _: bool, _: bool) {}

    fn config(&mut self, _: &mut Game, _: &mut ClientHandle, _: &[u8]) {}

    fn event(&mut self, _: &mut Game, _: &mut ClientHandle, _: ConnectionID, _: SharedEvent) {}

    fn tick_before(&mut self, _: &mut Game, _: &mut ClientHandle) {}

    fn tick_entity_before(
        &mut self, _: &mut Game, _: &mut Renderer, _: &ClientLevel,
        _: &mut ClientEntity, _: u8, _: f32
    ) {
    }

    fn tick_entity_after(
        &mut self, _: &mut Game, _: &mut Renderer, _: &ClientLevel,
        _: &mut ClientEntity, _: u8, _: f32
    ) {
    }

    fn tick_after(&mut self, _: &mut Game, _: &mut ClientHandle) {}

    fn draw(&mut self, _: &mut Game, _: &mut ClientHandle) {}

    fn destroy(&mut self, _: &mut Game, _: &mut ClientHandle) {}

    fn pop(&mut self, _: &mut Game, _: &mut ClientHandle) {}

}

