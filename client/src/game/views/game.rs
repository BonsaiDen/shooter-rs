// External Dependencies ------------------------------------------------------
use std::net::SocketAddr;


// Internal Dependencies ------------------------------------------------------
use shared::Lithium::Cobalt::ConnectionID;
use renderer::AllegroRenderer;
use level::RenderedLevel;
use game::{Game, ClientHandle, ClientEntity, ClientLevel};
use shared::Lithium::{EntityInput, EntityState, ClientHandler};
use shared::{Color, ColorName, SharedEvent, SharedCommand};
use self::super::{View, MenuView};


// View Implementation --------------------------------------------------------
#[derive(Debug)]
pub struct GameView {
    server_addr: SocketAddr
}

impl GameView {
    pub fn new(server_addr: SocketAddr) -> GameView {
        GameView {
            server_addr: server_addr
        }
    }
}

impl View for GameView {

    fn name(&self) -> &str {
        "Init"
    }

    fn push(&mut self, _: &mut Game, client: &mut ClientHandle) {
        client.events.send(SharedEvent::JoinGame);
    }

    fn config(&mut self, game: &mut Game, mut client: &mut ClientHandle, level_data: &[u8]) {
        client.level.set(RenderedLevel::from_serialized(level_data));
        game.reset(&mut client);
    }

    fn disconnect(&mut self, game: &mut Game, _: &mut ClientHandle, _: bool) {
        println!("[Client] Connection lost / closed."); // TODO indicate this via argument
        game.set_view(Box::new(MenuView));
    }

    fn event(&mut self, _: &mut Game, _: &mut ClientHandle, owner: ConnectionID, event: SharedEvent) {
        println!("[Client] Event: {:?} {:?}", owner, event);
    }

    fn tick_before(&mut self, _: &mut Game, client: &mut ClientHandle) {
        let tick = client.entities.tick();
        client.renderer.reseed_rng([
            ((tick as u32 + 7) * 941) as u32,
            ((tick as u32 + 659) * 461) as u32,
            ((tick as u32 + 13) * 227) as u32,
            ((tick as u32 + 97) * 37) as u32
        ]);
    }

    fn tick_entity_before(
        &mut self,
        _: &mut Game,
        renderer: &mut AllegroRenderer,
        _: &ClientLevel,
        entity: &mut ClientEntity,
        tick: u8, _: f32
    ) {

        if entity.local() {

            let mut buttons = 0;
            if renderer.key_down(1) || renderer.key_down(82) {
                buttons |= 0x01;
            }

            if renderer.key_down(4) || renderer.key_down(83) {
                buttons |= 0x02;
            }

            if renderer.key_down(23) || renderer.key_down(84) {
                buttons |= 0x04;
            }

            entity.local_input(EntityInput {
                tick: tick,
                fields: buttons
            });

        }

    }

    fn draw(&mut self, _: &mut Game, client: &mut ClientHandle) {

        client.renderer.clear(&Color::from_name(ColorName::Black));
        client.level.draw(client.renderer);
        client.entities.draw(client.renderer, client.level);
        client.renderer.draw_particles();

        let network_state = format!(
            "Press ESC to return to Menu - {}\nPing: {}ms - sent/recv: {}/{} - loss: {:.2}%",
            self.server_addr,
            client.network.rtt() / 2,
            client.network.bytes_sent(),
            client.network.bytes_received(),
            client.network.packet_loss()
        );

        client.renderer.text(
            &Color::from_name(ColorName::White),
            0.0, 0.0,
            &network_state[..]
        );

        if client.renderer.key_released(59) {
            client.events.send(SharedEvent::LeaveGame);
            client.events.send(SharedEvent::Command(SharedCommand::Shutdown));
        }

    }

    fn destroy(&mut self, _: &mut Game, client: &mut ClientHandle) {
        client.events.send(SharedEvent::Command(SharedCommand::Shutdown));
    }

}

