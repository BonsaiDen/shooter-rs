// External Dependencies ------------------------------------------------------
use lithium::entity;
use lithium::renderer::Renderer;
use lithium::client::{Handle, Handler};
use lithium::level::Level as LithiumLevel;
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
use game::{Game, State};
use shared::event::Event;
use shared::level::Level;
use renderer::AllegroRenderer;
use shared::color::{Color, ColorName};


// Handler Implementation -----------------------------------------------------
impl Handler<Event, Level> for Game {

    fn init(&mut self, client: Handle<Event, Level>) {

        let ar = AllegroRenderer::downcast_mut(client.renderer);
        ar.set_fps(60);
        ar.set_title("Rustgame: Shooter");
        ar.resize(client.level.width() as i32, client.level.height() as i32);

        // Local Test Play
        if self.state == State::Disconnected {

            let (x, y) = client.level.center();
            let flags = 0b0000_0001 | Color::from_name(ColorName::Red).to_flags();
            let state = entity::State {
                x: x as f32,
                y: y as f32,
                flags: flags,
                .. entity::State::default()
            };

            client.entities.create(0, Some(state), None);

        }

    }

    fn level(&mut self, _: Handle<Event, Level>, level_data: &[u8]) -> Level {
        Level::from_serialized(level_data)
    }

    fn config(&mut self, client: Handle<Event, Level>) {
        self.state = State::Connected;
        self.init(client);
    }

    fn connect(&mut self, server: Handle<Event, Level>) {
        self.state = State::Pending;
        server.events.send(None, Event::JoinGame);
    }

    fn disconnect(&mut self, client: Handle<Event, Level>) {
        self.state = State::Disconnected;
        self.init(client);
    }

    fn event(&mut self, _: Handle<Event, Level>, owner: ConnectionID, event: Event) {
        println!("Event: {:?} {:?}", owner, event);
    }

    fn tick_before(&mut self, client: Handle<Event, Level>, tick: u8, _: f32) {

        let ar = AllegroRenderer::downcast_mut(client.renderer);
        ar.reseed_rng([
            ((tick as u32 + 7) * 941) as u32,
            ((tick as u32 + 659) * 461) as u32,
            ((tick as u32 + 13) * 227) as u32,
            ((tick as u32 + 97) * 37) as u32
        ]);

    }

    fn tick_entity_before(
        &mut self,
        renderer: &mut Renderer,
        _: &Level,
        entity: &mut entity::Entity,
        tick: u8, _: f32
    ) {

        let ar = AllegroRenderer::downcast_mut(renderer);

        if entity.local() {

            let mut buttons = 0;
            if ar.key_down(1) || ar.key_down(82) {
                buttons |= 0x01;
            }

            if ar.key_down(4) || ar.key_down(83) {
                buttons |= 0x02;
            }

            if ar.key_down(23) || ar.key_down(84) {
                buttons |= 0x04;
            }

            let input = entity::Input {
                tick: tick,
                fields: buttons
            };

            entity.local_input(input);

        }

    }

    fn tick_entity_after(
        &mut self,
        _: &mut Renderer,
        _: &Level,
        entity: &mut entity::Entity,
        _: u8, _: f32

    ) -> entity::ControlState {

        if entity.local() {
            // TODO clean up once we have a local network proxy
            match self.state {
                State::Disconnected => entity::ControlState::Local,
                State::Connected => entity::ControlState::Remote,
                _ => entity::ControlState::None
            }

        } else {
           entity::ControlState::None
        }

    }

    fn tick_after(&mut self, _: Handle<Event, Level>, _: u8, _: f32) {
    }

    fn draw(&mut self, client: Handle<Event, Level>) {

        AllegroRenderer::downcast_mut(client.renderer).clear(&Color::from_name(ColorName::Black));

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

            AllegroRenderer::downcast_mut(client.renderer).text(
                &Color::from_name(ColorName::White), 0.0, 0.0, &network_state[..]
            );

        }

    }

    fn destroy(&mut self, _: Handle<Event, Level>) {

    }

}

