use rand::{SeedableRng, XorShiftRng};

use allegro::{Core, Color};
use allegro_primitives::PrimitivesAddon;
use allegro_font::{Font, FontAlign, FontDrawing};

use shared;
use shared::entity::{Entity, EntityInput, EntityState, EntityItem};
use shared::entities;
use shared::drawable::Drawable;
use net::{Network, MessageKind};

pub struct Game {
    back_color: Color,
    text_color: Color,
    rng: XorShiftRng,
    entities: Vec<EntityItem>,
    remote_states: Vec<(u8, EntityState)>,
    arena: shared::arena::Arena
}

impl Game {

    pub fn new(core: &Core) -> Game {
        Game {
            back_color: core.map_rgb(0, 0, 0),
            text_color: core.map_rgb(255, 255, 255),
            rng: XorShiftRng::new_unseeded(),
            entities: Vec::new(),
            remote_states: Vec::new(),
            arena: shared::arena::Arena::new(768, 768, 16)
        }
    }

    pub fn init(&mut self, core: &Core) {

        let mut player_ship = self.entity_from_kind(0);
        let flags = 0b0000_0001;
        player_ship.1.flags(0, flags);
        player_ship.0.set_state(EntityState {
            x: 400.0,
            y: 400.0,
            flags: flags,
            .. EntityState::default()
        });

        self.add_entity(player_ship);

    }


    // Networking -------------------------------------------------------------
    pub fn connect(&mut self, core: &Core) {
        self.reset();
    }

    pub fn disconnect(&mut self, core: &Core) {
        self.reset();
        self.init(core);
    }

    pub fn tick(
        &mut self, network: &mut Network, &
        key_state: &[bool; 255], initial_tick: bool, tick: u8, dt: f32
    ) {

        for &mut(ref mut e, _, _) in self.entities.iter_mut() {

            if e.is_local() {

                let input = EntityInput {
                    tick: tick as u8,
                    left: key_state[1] || key_state[82],
                    right: key_state[4] || key_state[83],
                    thrust: key_state[23] || key_state[84],
                    fire: false
                };

                e.input(input);

                // Emulate remote server state stuff with a 20 frames delay
                if self.remote_states.len() > 20 {
                    let first = self.remote_states.remove(0);
                    e.remote_tick(&self.arena, dt, initial_tick, first.0, first.1);

                } else {
                    e.tick(&self.arena, dt, initial_tick);
                }

                self.remote_states.push((tick, e.get_state()));

                // Send all unconfirmed inputs to server
                let mut input_buffer = Vec::<u8>::new();
                e.serialize_inputs(&mut input_buffer);
                network.send(MessageKind::Instant, input_buffer);

            } else {
                e.tick(&self.arena, dt, initial_tick);
            }

        }

        self.rng.reseed([
            ((tick as u32 + 7) * 941) as u32,
            ((tick as u32 + 659) * 461) as u32,
            ((tick as u32 + 13) * 227) as u32,
            ((tick as u32 + 97) * 37) as u32
        ]);

    }

    pub fn state(&mut self, data: &[u8]) {

        // Mark all entities as dead
        for &mut(_, _, mut alive) in self.entities.iter_mut() {
            alive = false;
        }

        let mut i = 0;
        while i < data.len() {

            // TODO check number of bytes left

            // Entity ID
            let id = data[i] as u32; // TODO 255 entity limit?

            // Entity Kind
            let kind = data[i + 1];

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
        for &mut (ref mut e, ref mut d, alive) in self.entities.iter_mut() {
            if alive == false {
                e.destroy();
                d.destroy();
            }
        }

        // ...then remove them from the list
        self.entities.retain(|&(_, _, active)| active);

    }


    // Rendering --------------------------------------------------------------
    pub fn draw(
        &mut self, core: &Core, prim: &PrimitivesAddon, font: &Font,
        network: &mut Network,
        dt: f32, u: f32
    ) {

        core.clear_to_color(self.back_color);

        // Draw all entities
        for &mut(ref mut e, ref mut d, _) in self.entities.iter_mut() {
            d.draw(&core, &prim, &mut self.rng, &self.arena, &**e, dt, u);
        }

        // UI
        let network_state = match network.connected() {
            true => format!("Connected to server at {}", network.server_addr()),
            false => format!("Connecting to server at {}...", network.server_addr())
        };
        core.draw_text(font, self.text_color, 0.0, 0.0, FontAlign::Left, &network_state[..]);

    }


    // Internal ---------------------------------------------------------------
    fn entity_from_kind(&self, kind: u8) -> EntityItem {
        match kind {
            0 => entities::ship::Ship(1.0),
            _ => unreachable!()
        }
    }

    fn add_entity(&mut self, mut entity: EntityItem) {
        self.entities.push(entity);
    }

    fn reset(&mut self) {
        self.remote_states.clear();
        self.entities.clear();
    }

}

