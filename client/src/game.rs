use rand::{SeedableRng, XorShiftRng};
use std::collections::HashMap;

use net::{Network, MessageKind};

use entities;
use shared::entity;
use shared::color::{Color, ColorName};
use shared::level::Level;
use shared::renderer::Renderer;

enum GameState {
    Disconnected,
    Pending,
    Connected
}

pub struct Game {
    back_color: Color,
    text_color: Color,
    rng: XorShiftRng,
    entities: HashMap<u16, entity::Entity>,
    remote_states: Vec<(u8, entity::State)>,
    level: Level,
    state: GameState
}

impl Game {

    pub fn new() -> Game {
        Game {
            back_color: Color::from_name(ColorName::Black),
            text_color: Color::from_name(ColorName::White),
            rng: XorShiftRng::new_unseeded(),
            entities: HashMap::new(),
            remote_states: Vec::new(),
            level: Level::new(768, 768, 16),
            state: GameState::Disconnected
        }
    }

    pub fn init(&mut self, renderer: &mut Renderer, level: Level, remote: bool) {

        // Local Test Play
        if remote == false {

            let mut player_ship = entity_from_kind(0);

            let (x, y) = level.center();
            let flags = 0b0000_0001 | Color::from_name(ColorName::Red).to_flags();
            player_ship.set_state(entity::State {
                x: x as f32,
                y: y as f32,
                flags: flags,
                .. entity::State::default()
            });

            self.add_entity(player_ship);

        }

        self.level = level;

        renderer.resize(self.level.width() as i32, self.level.height() as i32);

    }


    // Networking -------------------------------------------------------------

    pub fn connect(&mut self) {
        self.state = GameState::Pending;
        self.reset();
    }

    pub fn tick(
        &mut self, network: &mut Network, &
        key_state: &[bool; 255], tick: u8, dt: f32
    ) {

        for (_, entity) in self.entities.iter_mut() {

            if entity.local() {

                let input = entity::Input {
                    tick: tick as u8,
                    left: key_state[1] || key_state[82],
                    right: key_state[4] || key_state[83],
                    thrust: key_state[23] || key_state[84],
                    fire: false
                };

                let pending_input = entity.local_input(input);
                entity.client_tick(&self.level, tick, dt);

                // Emulate remote server state stuff with a 20 frames delay
                match self.state {

                    GameState::Disconnected => {

                        self.remote_states.push((tick, entity.get_state()));

                        if self.remote_states.len() > 20 {
                            let first = self.remote_states.remove(0);
                            entity.set_confirmed_state(first.0, first.1);
                        }

                    },

                    // Send all unconfirmed inputs to server
                    GameState::Connected => {
                        // TODO create a fake local network proxy!
                        network.send_message(MessageKind::Instant, pending_input);
                    },

                    _ => {}
                }


            } else {
                entity.client_tick(&self.level, tick, dt);
            }

        }

        self.rng.reseed([
            ((tick as u32 + 7) * 941) as u32,
            ((tick as u32 + 659) * 461) as u32,
            ((tick as u32 + 13) * 227) as u32,
            ((tick as u32 + 97) * 37) as u32
        ]);

    }

    pub fn message(
        &mut self,
        renderer: &mut Renderer, kind: u8, data: &[u8],
        tick: u8

    ) -> u8 {
        match self.state {
            GameState::Pending => {

                // Game Configuration
                if kind == 0 {
                    // TODO validate message length
                    self.config(renderer, data);
                }

                tick

            },
            GameState::Connected => {
                // Game State
                if kind == 1 {
                    // TODO validate message length?
                    let confirmed_input_tick = data[1];
                    //println!("confirmed input tick {} (remote: {}, local: {})", data[1], data[0], tick);
                    self.state(&data[2..], tick, confirmed_input_tick);
                    tick

                } else {
                    tick
                }
            },
            _ => tick
        }
    }

    pub fn disconnect(&mut self, renderer: &mut Renderer, level: Level) {
        self.state = GameState::Disconnected;
        self.reset();
        self.init(renderer, level, false);
    }


    // Rendering --------------------------------------------------------------
    pub fn draw(
        &mut self,
        renderer: &mut Renderer,
        network: &mut Network,
        dt: f32, u: f32
    ) {

        renderer.clear(&self.back_color);

        // Draw all entities
        for (_, entity) in self.entities.iter_mut() {
            entity.draw(
                renderer, &mut self.rng,
                &self.level, dt, u
            );
        }

        // UI
        if let Ok(addr) = network.server_addr() {
            let network_state = match network.connected() {
                true => format!(
                    "{} (Ping: {}ms, Lost: {}%, Bytes: {}/{})",
                    addr,
                    network.rtt() / 2,
                    network.packet_loss(),
                    network.bytes_sent(),
                    network.bytes_received()
                ),
                false => format!("Connecting to {}...", addr)
            };
            renderer.text(&self.text_color, 0.0, 0.0, &network_state[..]);
        }

        renderer.draw(dt, u);

    }


    // Internal ---------------------------------------------------------------
    fn add_entity(&mut self, entity: entity::Entity) {
        self.entities.insert(entity.id(), entity);
    }

    fn reset(&mut self) {
        self.remote_states.clear();
        self.entities.clear();
    }

    fn config(&mut self, renderer: &mut Renderer, data: &[u8]) {
        self.init(renderer, Level::from_serialized(&data[0..5]), true);
        self.state = GameState::Connected;
    }

    fn state(&mut self, data: &[u8], tick: u8, confirmed_tick: u8) {

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
                    entity.set_state(state);
                    entity.client_created(tick);
                    entity
                });

                // Mark entity as alive
                entity.set_alive(true);

                // Set Remote state
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

