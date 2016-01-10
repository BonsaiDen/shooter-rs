use rand::{SeedableRng, XorShiftRng};
use std::collections::HashMap;

use allegro::{Core, Color as AllegroColor, Display};
use allegro_primitives::PrimitivesAddon;
use allegro_font::{Font, FontAlign, FontDrawing};

use net::{Network, MessageKind};

use entities;
use shared::color::Color;
use shared::arena::Arena;
use shared::entity;
use shared::drawable::Drawable;
use shared::particle::ParticleSystem;

enum GameState {
    Disconnected,
    Pending,
    Connected
}

pub struct Game {
    back_color: AllegroColor,
    text_color: AllegroColor,
    rng: XorShiftRng,
    entities: HashMap<u16, entity::Entity>,
    remote_states: Vec<(u8, entity::State)>,
    arena: Arena,
    particle_system: ParticleSystem,
    state: GameState
}

impl Game {

    pub fn new(_: &Core) -> Game {
        Game {
            back_color: AllegroColor::from_rgb(0, 0, 0),
            text_color: AllegroColor::from_rgb(255, 255, 255),
            rng: XorShiftRng::new_unseeded(),
            entities: HashMap::new(),
            remote_states: Vec::new(),
            arena: Arena::new(768, 768, 16),
            state: GameState::Disconnected,
            particle_system: ParticleSystem::new(1000),
        }
    }

    pub fn init(&mut self, _: &Core, disp: &mut Display, arena: Arena, remote: bool) {

        // Local Test Play
        if remote == false {

            let mut player_ship = entity_from_kind(0);

            let (x, y) = arena.center();
            let flags = 0b0000_0001 | Color::Red.to_flags();
            player_ship.set_state(entity::State {
                x: x as f32,
                y: y as f32,
                flags: flags,
                .. entity::State::default()
            });

            self.add_entity(player_ship);

        }

        self.arena = arena;

        // TODO handle errors or ignore?
        disp.resize(self.arena.width() as i32, self.arena.height() as i32).ok();

    }


    // Networking -------------------------------------------------------------

    pub fn connect(&mut self, _: &Core) {
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

                entity.input(input, 30); // TODO set tick rate externally

                // Emulate remote server state stuff with a 20 frames delay
                match self.state {
                    GameState::Disconnected => {
                        if self.remote_states.len() > 20 {
                            let first = self.remote_states.remove(0);
                            entity.tick_remote(&self.arena, dt, first.0, first.1);

                        } else {
                            entity.tick_local(&self.arena, dt, true);
                        }

                        self.remote_states.push((tick, entity.get_state()));

                    },

                    GameState::Connected => {

                        entity.tick_local(&self.arena, dt, true);

                        // Send all unconfirmed inputs to server
                        network.send_message(MessageKind::Instant, entity.serialize_inputs());

                    },

                    _ => {}
                }


            } else {
                entity.tick_local(&self.arena, dt, true);
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
        core: &Core, disp: &mut Display, kind: u8, data: &[u8],
        tick: u8,
        dt: f32

    ) -> u8 {
        match self.state {
            GameState::Pending => {

                // Game Configuration
                if kind == 0 {
                    // TODO validate message length
                    self.config(core, disp, data);
                }

                tick

            },
            GameState::Connected => {
                // Game State
                if kind == 1 {
                    // TODO validate message length?
                    let remote_tick = data[0];
                    self.state(&data[1..], tick, remote_tick, dt);
                    remote_tick

                } else {
                    tick
                }
            },
            _ => tick
        }
    }

    pub fn disconnect(&mut self, core: &Core, disp: &mut Display, arena: Arena) {
        self.state = GameState::Disconnected;
        self.reset();
        self.init(core, disp, arena, false);
    }


    // Rendering --------------------------------------------------------------
    pub fn draw(
        &mut self, core: &Core, prim: &PrimitivesAddon, font: &Font,
        network: &mut Network,
        dt: f32, u: f32
    ) {

        core.clear_to_color(self.back_color);

        // Draw all entities
        for (_, entity) in self.entities.iter_mut() {
            entity.draw(
                &core, &prim, &mut self.rng, &mut self.particle_system,
                &self.arena, dt, u
            );
        }

        self.particle_system.draw(&prim, dt);

        // UI
        if let Ok(addr) = network.server_addr() {
            let network_state = match network.connected() {
                true => format!(
                    "Connected to {} (Ping: {}ms, Packet Loss: {}%)",
                    addr,
                    network.rtt() / 2,
                    network.packet_loss()
                ),
                false => format!("Connecting to {}...", addr)
            };
            core.draw_text(font, self.text_color, 0.0, 0.0, FontAlign::Left, &network_state[..]);
        }

    }


    // Internal ---------------------------------------------------------------
    fn add_entity(&mut self, entity: entity::Entity) {
        self.entities.insert(entity.id(), entity);
    }

    fn reset(&mut self) {
        self.remote_states.clear();
        self.entities.clear();
    }

    fn config(&mut self, core: &Core, disp: &mut Display, data: &[u8]) {
        println!("Received Config"); // TODO update tick rate from config?
        self.init(core, disp, Arena::from_serialized(&data[0..5]), true);
        self.state = GameState::Connected;
    }

    fn state(&mut self, data: &[u8], tick: u8, remote_tick: u8, dt: f32) {

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
                    entity.set_id(id);
                    entity.set_state(state);
                    entity.create();
                    entity
                });

                // Mark entity as alive
                entity.set_alive(true);

                // Update Entity State
                if entity.local() {
                    entity.tick_remote(&self.arena, dt, remote_tick, state);

                } else {
                    entity.set_state(state);
                }

            }

            // TODO trigger flag() method on entity if any flag changed

        }

        // Destroy dead entities...
        let mut destroyed_ids = Vec::new();
        for (_, entity) in self.entities.iter_mut() {
            if entity.alive() == false {
                entity.destroy();
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

