// External Dependencies ------------------------------------------------------
use cobalt::ConnectionID;
use lithium::{
    Entity,
    EntityInput,
    EntityState,
    ClientHandle,
    ClientHandler,
    Level,
    Renderer
};


// Internal Dependencies ------------------------------------------------------
use level::RenderedLevel;
use game::{Game, GameState};
use renderer::AllegroRenderer;
use shared::{Color, ColorName, SharedEvent, SharedCommand, SharedLevel, SharedState};


// Type Aliases ---------------------------------------------------------------
pub type Handle<'a> = ClientHandle<'a, SharedEvent, SharedState, SharedLevel, AllegroRenderer>;
pub type ClientEntity = Entity<SharedState, SharedLevel, AllegroRenderer>;
pub type ClientLevel = Level<SharedState, SharedLevel>;


// Handler Implementation -----------------------------------------------------
impl ClientHandler<SharedEvent, SharedState, SharedLevel, AllegroRenderer> for Game {

    fn init(&mut self, client: Handle) {
        client.renderer.set_fps(60);
        client.renderer.set_title("Rustgame: Shooter");
        client.renderer.resize(client.level.width() as i32, client.level.height() as i32);
    }

    fn config(&mut self, client: Handle, level_data: &[u8]) {
        client.level.set(RenderedLevel::from_serialized(level_data));
        self.state = GameState::Connected;
        self.init(client);
    }

    fn connect(&mut self, client: Handle) {
        self.state = GameState::Pending;
        client.events.send(None, SharedEvent::JoinGame);
    }

    fn disconnect(&mut self, client: Handle, was_connected: bool) {

        self.state = GameState::Disconnected;
        self.last_connection_retry = client.renderer.time();

        if was_connected {
            self.init(client);
            println!("[Client] Connection lost.");

        } else {
            println!("[Client] Connection failed.");
        }

    }

    fn event(&mut self, _: Handle, owner: ConnectionID, event: SharedEvent) {
        println!("[Client] Event: {:?} {:?}", owner, event);
    }

    fn tick_before(&mut self, client: Handle) {

        // Retry Connections
        let timeout = client.renderer.time() - self.last_connection_retry;
        if self.state == GameState::Disconnected && timeout > 3.0 {
            println!("[Client] Establishing connection...");
            self.last_connection_retry = 0.0;
            client.network.reset();
        }

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

            let input = EntityInput {
                tick: tick,
                fields: buttons
            };

            entity.local_input(input);

        }

    }

    fn tick_entity_after(
        &mut self,
        _: &mut AllegroRenderer,
        _: &ClientLevel,
        _: &mut ClientEntity,
        _: u8, _: f32
    ) {

    }

    fn tick_after(&mut self, _: Handle) {
    }

    fn draw(&mut self, client: Handle) {

        client.renderer.clear(&Color::from_name(ColorName::Black));

        client.level.draw(client.renderer);

        client.entities.draw(client.renderer, client.level);

        if let Ok(addr) = client.network.server_addr() {
            let network_state = match self.state {
                GameState::Connected => format!(
                    "{} (Ping: {}ms, Lost: {:.2}%, Bytes: {}/{})",
                    addr,
                    client.network.rtt() / 2,
                    client.network.packet_loss(),
                    client.network.bytes_sent(),
                    client.network.bytes_received()
                ),
                _ => format!("Connecting to {}...", addr)
            };

            client.renderer.text(
                &Color::from_name(ColorName::White),
                0.0, 0.0,
                &network_state[..]
            );

        }

    }

    fn destroy(&mut self, client: Handle) {
        client.events.send(None, SharedEvent::Command(SharedCommand::Shutdown));
    }

}

