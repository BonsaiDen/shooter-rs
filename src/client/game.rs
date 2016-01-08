use rand::{SeedableRng, XorShiftRng};

use allegro::{Core, Color as AllegroColor, Display};
use allegro_primitives::PrimitivesAddon;
use allegro_font::{Font, FontAlign, FontDrawing};

use net::{Network, MessageKind};

use shared::color::Color;
use shared::arena::Arena;
use shared::entity::{EntityType, EntityInput, EntityState, Entity};
use shared::entities;
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
    entities: Vec<Entity>,
    remote_states: Vec<(u8, EntityState)>,
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
            entities: Vec::new(),
            remote_states: Vec::new(),
            arena: Arena::new(768, 768, 16),
            state: GameState::Disconnected,
            particle_system: ParticleSystem::new(1000),
        }
    }

    pub fn init(&mut self, _: &Core, disp: &mut Display, arena: Arena, remote: bool) {

        // Local Test Play
        if remote == false {

            let mut player_ship = self.entity_from_kind(0);

            let (x, y) = arena.center();
            let flags = 0b0000_0001 | Color::Red.to_flags();
            player_ship.drawable.set_flags(0, flags);
            player_ship.typ.set_state(EntityState {
                x: x as f32,
                y: y as f32,
                flags: flags,
                .. EntityState::default()
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

        for entity in self.entities.iter_mut() {

            if entity.typ.is_local() {

                let input = EntityInput {
                    tick: tick as u8,
                    left: key_state[1] || key_state[82],
                    right: key_state[4] || key_state[83],
                    thrust: key_state[23] || key_state[84],
                    fire: false
                };

                entity.typ.input(input);

                // Emulate remote server state stuff with a 20 frames delay
                if self.remote_states.len() > 20 {
                    let first = self.remote_states.remove(0);
                    entity.typ.remote_tick(&self.arena, dt, first.0, first.1);

                } else {
                    entity.typ.tick(&self.arena, dt);
                }

                self.remote_states.push((tick, entity.typ.get_state()));

                // Send all unconfirmed inputs to server
                let mut input_buffer = Vec::<u8>::new();
                entity.typ.serialize_inputs(&mut input_buffer);
                network.send_message(MessageKind::Instant, input_buffer);

            } else {
                entity.typ.tick(&self.arena, dt);
            }

        }

        self.rng.reseed([
            ((tick as u32 + 7) * 941) as u32,
            ((tick as u32 + 659) * 461) as u32,
            ((tick as u32 + 13) * 227) as u32,
            ((tick as u32 + 97) * 37) as u32
        ]);

    }

    pub fn message(&mut self, core: &Core, disp: &mut Display, kind: u8, data: &[u8]) {
        match self.state {
            GameState::Pending => {

                // Game Configuration
                if kind == 0 {
                    // TODO validate message length
                    self.config(core, disp, data);
                }

            },
            GameState::Connected => {
                // Game State
                if kind == 1 {
                    // TODO validate message length?
                    self.state(data);
                }
            },
            _ => ()
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
        for entity in self.entities.iter_mut() {
            entity.drawable.draw(
                &core, &prim, &mut self.rng, &mut self.particle_system,
                &self.arena, &*entity.typ, dt, u
            );
        }

        self.particle_system.draw(&core, &prim, dt);

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
    fn entity_from_kind(&self, kind: u8) -> Entity {
        match kind {
            0 => entities::ship::DrawableShip(1.0),
            _ => unreachable!()
        }
    }

    fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    fn reset(&mut self) {
        self.remote_states.clear();
        self.entities.clear();
    }

    fn config(&mut self, core: &Core, disp: &mut Display, data: &[u8]) {
        println!("Received Config");
        self.init(core, disp, Arena::from_serialized(&data[0..5]), true);
        self.state = GameState::Connected;
    }

    fn state(&mut self, data: &[u8]) {

        println!("Received state");

        // Mark all entities as dead
        for entity in self.entities.iter_mut() {
            entity.set_alive(false);
        }

        let mut i = 0;
        while i < data.len() {

            // TODO check number of bytes left

            // Entity ID
            let id = data[i] as u32; // TODO use 16 bit

            // Entity Kind
            let kind = data[i + 1];

            // TODO index based on id AND kind to improve hashing

            // TODO create entites which are not yet in the map
            //let entity = if entity_map.contains_key(&id) == false {
            //    //self.add_entity(self.entity_from_kind(kind));
            //    // TODO call create method on entity

            //} else {
            //    self.entity_map.get_mut(&id);
            //};

            // TODO Mark entity as alive

            // Update Entity State

            // TODO if entity is local apply remote tick to entity
            // TODO otherwise simply apply state directly

            // TODO trigger create() method on entity
            // TODO trigger flag() method on entity if any flag changed

            i += 1;

        }

        // Destroy dead entities...
        for entity in self.entities.iter_mut() {
            if entity.alive() == false {
                entity.typ.destroy();
                entity.drawable.destroy();
            }
        }

        // ...then remove them from the list
        self.entities.retain(|ref entity| entity.alive());

    }

}

