// External Dependencies ------------------------------------------------------
use cobalt::ConnectionID;
use lithium::{
    Entity,
    EntityInput,
    EntityState,
    EntityControlState,
    ClientHandle,
    ClientHandler,
    Level,
    Renderer
};


// Internal Dependencies ------------------------------------------------------
use level::RenderedLevel;
use game::{Game, GameState};
use shared::{Color, ColorName, SharedEvent, SharedLevel, SharedState};
use renderer::AllegroRenderer;


// Type Aliases ---------------------------------------------------------------
type Handle<'a> = ClientHandle<'a, SharedEvent, SharedState, SharedLevel, AllegroRenderer>;
type ClientEntity = Entity<SharedState, SharedLevel, AllegroRenderer>;
type ClientLevel = Level<SharedState, SharedLevel>;


// Handler Implementation -----------------------------------------------------
impl ClientHandler<SharedEvent, SharedState, SharedLevel, AllegroRenderer> for Game {

    fn init(&mut self, client: Handle) {

        client.renderer.set_fps(60);
        client.renderer.set_title("Rustgame: Shooter");
        client.renderer.resize(client.level.width() as i32, client.level.height() as i32);

        // Local Test Play
        if self.state == GameState::Disconnected {

            let (x, y) = client.level.center();
            let flags = 0b0000_0001 | Color::from_name(ColorName::Red).to_flags();
            let state = SharedState {
                x: x as f32,
                y: y as f32,
                flags: flags,
                .. SharedState::default() // TODO implement default so we can use Default::default()
            };

            client.entities.create(0, Some(state), None).unwrap().show(0);

        }

    }

    fn level(&mut self, _: Handle, level_data: &[u8]) -> Level<SharedState, SharedLevel> {
        RenderedLevel::from_serialized(level_data)
    }

    fn config(&mut self, client: Handle) {
        self.state = GameState::Connected;
        self.init(client);
    }

    fn connect(&mut self, server: Handle) {
        self.state = GameState::Pending;
        server.events.send(None, SharedEvent::JoinGame);
    }

    fn disconnect(&mut self, client: Handle) {
        self.state = GameState::Disconnected;
        self.init(client);
    }

    fn event(&mut self, _: Handle, owner: ConnectionID, event: SharedEvent) {
        println!("Event: {:?} {:?}", owner, event);
    }

    fn tick_before(&mut self, client: Handle, tick: u8, _: f32) {
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
        entity: &mut ClientEntity,
        _: u8, _: f32

    ) -> EntityControlState {

        // TODO have a method on the entity?
        if entity.local() {
            // TODO clean up once we have a local network proxy
            match self.state {
                GameState::Disconnected => EntityControlState::Local,
                GameState::Connected => EntityControlState::Remote,
                _ => EntityControlState::None
            }

        } else {
           EntityControlState::None
        }

    }

    fn tick_after(&mut self, _: Handle, _: u8, _: f32) {
    }

    fn draw(&mut self, client: Handle) {

        client.renderer.clear(&Color::from_name(ColorName::Black));

        client.level.draw(client.renderer);

        client.entities.draw(client.renderer, client.level);

        if let Ok(addr) = client.network.server_addr() {
            let network_state = match client.network.connected() {
                true => format!(
                    "{} (Ping: {}ms, Lost: {:.2}%, Bytes: {}/{})",
                    addr,
                    client.network.rtt() / 2,
                    client.network.packet_loss(),
                    client.network.bytes_sent(),
                    client.network.bytes_received()
                ),
                false => format!("Connecting to {}...", addr)
            };

            client.renderer.text(
                &Color::from_name(ColorName::White),
                0.0, 0.0,
                &network_state[..]
            );

        }

    }

    fn destroy(&mut self, _: Handle) {

    }

}

