// External Dependencies ------------------------------------------------------
use std::net::SocketAddr;
use lithium::entity;
use lithium::renderer::Renderer;
use lithium::runnable::Runnable;


// Internal Dependencies ------------------------------------------------------
use net;
use entities;
use renderer::AllegroRenderer;
use shared::color::{Color, ColorName};
use shared::level::Level;

enum GameState {
    Disconnected,
    Pending,
    Connected
}

pub struct EntityRegistry;

impl entity::Registry for EntityRegistry {
    fn entity_from_type_id(&self, type_id: u8) -> entity::Entity {
        match type_id {
            0 => entities::Ship::create_entity(1.0),
            _ => unreachable!()
        }
    }
}

pub struct Game {
    back_color: Color,
    text_color: Color,
    network: net::Network,
    manager: entity::Manager,
    remote_states: Vec<(u8, entity::State)>,
    level: Level,
    state: GameState
}

impl Runnable for Game {

    fn init(&mut self, renderer: &mut Renderer) {

        // TODO clean up!
        self.network.set_tick_rate(self.manager.config().tick_rate as u32);
        self.manager.init(renderer);
        renderer.set_fps(60);

        let ar = AllegroRenderer::downcast_mut(renderer);
        ar.set_title("Rustgame: Shooter");
        ar.resize(self.level.width() as i32, self.level.height() as i32);

        // Local Test Play
        if self.network.connected() == false {

            let (x, y) = self.level.center();
            let flags = 0b0000_0001 | Color::from_name(ColorName::Red).to_flags();
            let state = entity::State {
                x: x as f32,
                y: y as f32,
                flags: flags,
                .. entity::State::default()
            };

            self.manager.create_entity(0, Some(state), None);

        }

    }

    fn tick(&mut self, renderer: &mut Renderer) -> bool {

        let mut ticked = false;
        let tick_rate = self.network.get_tick_rate();
        let dt = 1.0 / tick_rate as f32;

        self.network.receive();

        while let Ok(event) = self.network.message(renderer.time()) {
            match event {

                net::EventType::Connection(_) => {
                    self.connect();
                },

                net::EventType::Message(_, data) =>  {
                    // TODO validate message length
                    if data.len() > 0 {
                        match self.state {
                            GameState::Pending => {

                                // Game Configuration
                                // TODO use enum for type
                                if data[0] == 0 {
                                    self.config(renderer, &data[1..]);
                                }

                            },
                            GameState::Connected => {
                                // Game State
                                // TODO use enum for type
                                if data[0] == 1 {
                                    self.manager.receive_state(&data[1..]);
                                }
                            },
                            _ => {}
                        }
                    }
                },

                net::EventType::Tick(_, _, _) => {
                    ticked = true;
                    self.tick_entities(renderer, dt);
                },

                net::EventType::Close => {
                    println!("Connection closed");
                },

                net::EventType::ConnectionLost(_) => {
                    self.disconnect(renderer);
                },

                _ => {}

            }
        }

        self.network.send();

        ticked

    }

    fn draw(&mut self, renderer: &mut Renderer) {

        AllegroRenderer::downcast_mut(renderer).clear(&self.back_color);

        // Draw all entities
        self.manager.draw_entities(renderer, &self.level);

        // UI
        if let Ok(addr) = self.network.server_addr() {
            let network_state = match self.network.connected() {
                true => format!(
                    "{} (Ping: {}ms, Lost: {:.2}%, Bytes: {}/{})",
                    addr,
                    self.network.rtt() / 2,
                    self.network.packet_loss(),
                    self.network.bytes_sent(),
                    self.network.bytes_received()
                ),
                false => format!("Connecting to {}...", addr)
            };

            AllegroRenderer::downcast_mut(renderer).text(
                &self.text_color, 0.0, 0.0, &network_state[..]
            );

        }

    }

    fn destroy(&mut self) {
        self.network.destroy();
    }

}

impl Game {

    pub fn new(server_addr: SocketAddr) -> Game {
        Game {
            back_color: Color::from_name(ColorName::Black),
            text_color: Color::from_name(ColorName::White),
            manager: entity::Manager::new(
                30, 1000, 75,
                false,
                Box::new(EntityRegistry)
            ),
            remote_states: Vec::new(),
            network: net::Network::new(server_addr),
            level: Game::default_level(),
            state: GameState::Disconnected
        }
    }

    pub fn default_level() -> Level {
        Level::new(384, 384, 16)
    }


    // Internal ---------------------------------------------------------------
    fn connect(&mut self) {
        self.state = GameState::Pending;
        self.reset();
    }

    fn disconnect(&mut self, renderer: &mut Renderer) {
        self.level = Game::default_level();
        self.state = GameState::Disconnected;
        self.reset();
        self.init(renderer);
    }

    fn reset(&mut self) {
        self.remote_states.clear();
        self.manager.reset();
    }

    fn config(&mut self, renderer: &mut Renderer, data: &[u8]) {
        let level_data = self.manager.receive_config(renderer, data);
        self.network.set_tick_rate(self.manager.config().tick_rate as u32);
        self.level = Level::from_serialized(level_data);
        self.state = GameState::Connected;
        self.init(renderer);
    }

    fn tick_entities(&mut self, renderer: &mut Renderer, dt: f32) {

        let ar = AllegroRenderer::downcast_mut(renderer);
        let tick = self.manager.tick();
        ar.reseed_rng([
            ((tick as u32 + 7) * 941) as u32,
            ((tick as u32 + 659) * 461) as u32,
            ((tick as u32 + 13) * 227) as u32,
            ((tick as u32 + 97) * 37) as u32
        ]);

        let state = &self.state;
        let remote_states = &mut self.remote_states;
        let mut local_inputs = None;

        self.manager.tick_entities(&self.level, dt, |entity, _, tick, _| {

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

        }, |entity, _, tick, _| {

            if entity.local() {
                match *state {
                    GameState::Disconnected => {

                        // Emulate remote server state stuff with a 20 frames delay
                        remote_states.push((tick, entity.state().clone()));

                        if remote_states.len() > 20 {
                            let first = remote_states.remove(0);
                            entity.set_confirmed_state(first.0, first.1);
                        }

                    },

                    GameState::Connected => {
                        local_inputs = entity.serialized_inputs();
                    },

                    _ => {}

                }
            }

        });

        // Send all unconfirmed inputs to server
        if let Some(inputs) = local_inputs {
            // TODO create a fake local network proxy!
            self.network.send_message(net::MessageKind::Instant, inputs);
        }

    }

}

