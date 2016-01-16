use std::collections::HashMap;
use std::net::SocketAddr;

use net;
use entities;
use shared::entity;
use shared::color::{Color, ColorName};
use shared::level::Level;
use shared::renderer::{Renderer, Runnable};

enum GameState {
    Disconnected,
    Pending,
    Connected
}

pub struct Game {
    back_color: Color,
    text_color: Color,
    entities: HashMap<u16, entity::Entity>,
    remote_states: Vec<(u8, entity::State)>,
    tick: u8,
    network: net::Network,
    level: Level,
    state: GameState
}

impl Runnable for Game {

    fn init(&mut self, renderer: &mut Renderer) {

        renderer.set_title("Rustgame: Shooter");
        renderer.set_fps(60);
        renderer.set_tick_rate(self.network.get_tick_rate());
        renderer.set_interpolation_ticks(3);
        renderer.resize(self.level.width() as i32, self.level.height() as i32);

        // Local Test Play
        if self.network.connected() == false {

            let mut player_ship = entity_from_kind(0);

            let (x, y) = self.level.center();
            let flags = 0b0000_0001 | Color::from_name(ColorName::Red).to_flags();
            player_ship.set_state(entity::State {
                x: x as f32,
                y: y as f32,
                flags: flags,
                .. entity::State::default()
            });

            self.entities.insert(player_ship.id(), player_ship);

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
                                if data[0] == 0 {
                                    self.network.set_tick_rate(data[1] as u32);
                                    self.config(renderer, &data[2..]);
                                }

                            },
                            GameState::Connected => {
                                // Game State
                                if data[0] == 1 {
                                    self.state(&data[3..], data[2]);
                                }
                            },
                            _ => {}
                        }
                    }
                },

                net::EventType::Tick(_, _, _) => {

                    ticked = true;
                    self.tick_entities(renderer, dt);

                    if self.tick == 255 {
                        self.tick = 0;

                    } else {
                        self.tick += 1;
                    }

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

        renderer.clear(&self.back_color);

        // Draw all entities
        for (_, entity) in self.entities.iter_mut() {
            entity.draw(renderer, &self.level);
        }

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
            renderer.text(&self.text_color, 0.0, 0.0, &network_state[..]);
        }

        renderer.draw();

    }

    fn destroy(&mut self) {
        self.network.destroy();
    }

}

impl Game {

    pub fn default_level() -> Level {
        Level::new(384, 384, 16)
    }

    pub fn new(server_addr: SocketAddr) -> Game {
        Game {
            back_color: Color::from_name(ColorName::Black),
            text_color: Color::from_name(ColorName::White),
            entities: HashMap::new(),
            remote_states: Vec::new(),
            tick: 0,
            network: net::Network::new(server_addr),
            level: Game::default_level(),
            state: GameState::Disconnected
        }
    }

    // Internal ---------------------------------------------------------------

    fn tick_entities(&mut self, renderer: &mut Renderer, dt: f32) {

        for (_, entity) in self.entities.iter_mut() {

            if entity.local() {

                let input = entity::Input {
                    tick: self.tick,
                    left: renderer.key_down(1) || renderer.key_down(82),
                    right: renderer.key_down(4) || renderer.key_down(83),
                    thrust: renderer.key_down(23) || renderer.key_down(84),
                    fire: false
                };

                let pending_input = entity.local_input(input);
                entity.client_tick(&self.level, self.tick, dt);

                // Emulate remote server state stuff with a 20 frames delay
                match self.state {

                    GameState::Disconnected => {

                        self.remote_states.push((self.tick, entity.state().clone()));

                        if self.remote_states.len() > 20 {
                            let first = self.remote_states.remove(0);
                            entity.set_confirmed_state(first.0, first.1);
                        }

                    },

                    // Send all unconfirmed inputs to server
                    GameState::Connected => {
                        // TODO create a fake local network proxy!
                        self.network.send_message(
                            net::MessageKind::Instant, pending_input
                        );
                    },

                    _ => {}
                }


            } else {
                entity.client_tick(&self.level, self.tick, dt);
            }

        }

        renderer.reseed_rng([
            ((self.tick as u32 + 7) * 941) as u32,
            ((self.tick as u32 + 659) * 461) as u32,
            ((self.tick as u32 + 13) * 227) as u32,
            ((self.tick as u32 + 97) * 37) as u32
        ]);

    }

    fn connect(&mut self) {
        self.state = GameState::Pending;
        self.reset();
    }

    fn disconnect(&mut self, renderer: &mut Renderer) {
        self.tick = 0;
        self.level = Game::default_level();
        self.state = GameState::Disconnected;
        self.reset();
        self.init(renderer);
    }

    fn reset(&mut self) {
        self.remote_states.clear();
        self.entities.clear();
    }

    fn config(&mut self, renderer: &mut Renderer, data: &[u8]) {
        self.level = Level::from_serialized(&data[0..5]);
        self.state = GameState::Connected;
        self.init(renderer);
    }

    fn state(&mut self, data: &[u8], confirmed_tick: u8) {

        let tick = self.tick;

        // Mark all entities as dead
        for (_, entity) in self.entities.iter_mut() {
            entity.set_alive(false);
        }

        let mut i = 0;
        while i + 3 <= data.len() {

            // Entity ID / Kind
            let id = (data[i] as u16) << 8 | (data[i + 1] as u16);
            let kind = data[i + 2];
            i += 3;

            // Check serialized data
            if i + entity::State::encoded_size() <= data.len() {

                // Read serialized entity state
                let state = entity::State::from_serialized(&data[i..]);
                i += entity::State::encoded_size();

                // Create entities which do not yet exist
                let mut entity = self.entities.entry(id).or_insert_with(|| {
                    let mut entity = entity_from_kind(kind);
                    // TODO abstract away
                    entity.set_id(id);
                    entity.set_state(state.clone());
                    entity.client_created(tick);
                    entity
                });

                // Mark entity as alive
                entity.set_alive(true);

                // Set confirmed state
                if entity.local() {
                    entity.set_confirmed_state(confirmed_tick, state);

                // Or overwrite local state (but keep last_state intact for interpolation)
                } else {
                    entity.set_remote_state(state);
                }

            }

            // TODO trigger flag() method on entity if any flag changed?

        }

        // Destroy dead entities...
        let mut destroyed_ids = Vec::new();
        for (_, entity) in self.entities.iter_mut() {
            if entity.alive() == false {
                entity.client_destroyed(tick);
                destroyed_ids.push(entity.id());
            }
        }

        // ...then remove them from the map
        for id in &destroyed_ids {
            self.entities.remove(&id);
        }

    }

}

fn entity_from_kind(kind: u8) -> entity::Entity {
    match kind {
        0 => entities::Ship::create_entity(1.0),
        _ => unreachable!()
    }
}

