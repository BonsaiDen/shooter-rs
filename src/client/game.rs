use rand::{SeedableRng, XorShiftRng};
use std::collections::hash_map::HashMap;

use allegro::{Core, Color};
use allegro_primitives::PrimitivesAddon;
use allegro_font::{Font, FontAlign, FontDrawing};

use shared;
use shared::entity::{Entity, EntityState, EntityInput};
use net::{Network, MessageKind};

pub struct Game {
    back_color: Color,
    text_color: Color,
    rng: XorShiftRng,
    entity_id_map: HashMap<u32, bool>,
    entities: Vec<Box<Entity>>,
    remote_states: Vec<(u8, EntityState)>,
    arena: shared::arena::Arena
}

impl Game {

    pub fn new(core: &Core) -> Game {
        Game {
            back_color: core.map_rgb(0, 0, 0),
            text_color: core.map_rgb(255, 255, 255),
            rng: XorShiftRng::new_unseeded(),
            entity_id_map: HashMap::new(),
            entities: Vec::new(),
            remote_states: Vec::new(),
            arena: shared::arena::Arena::new(768, 768, 16)
        }
    }

    pub fn init(&mut self, core: &Core) {
        self.add_entity(Box::new(
            shared::ship::PlayerShip::new(
                60.0, 60.0, true, shared::color::Color::red()
            )
        ));
        // TODO implement network and events
        // self.network.send(NetworkEvent::JoinRequest()); ??
    }

    pub fn connect(&mut self, core: &Core) {
        self.reset();
    }

    pub fn disconnect(&mut self, core: &Core) {
        self.reset();
        self.init(core);
    }

    pub fn add_entity(&mut self, mut entity: Box<Entity>) {
        self.entity_id_map.insert(entity.get_id(), true);
        self.entities.push(entity);
    }

    pub fn tick(
        &mut self, network: &mut Network, &
        key_state: &[bool; 255], initial_tick: bool, tick: u8, dt: f32,
    ) {

        // TODO bullets are handled by pre-creating a local object and then
        // syncing it with the remote one, we submit a local ID and the server
        // return this ID along with the remote object ID when updating

        // TODO server side
        // - send full state or diff from last confirmed local tick?

        for e in self.entities.iter_mut() {

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
                    // TODO apply the states received from the server
                    let first = self.remote_states.remove(0);
                    e.remote_tick(&self.arena, dt, initial_tick, first.0, first.1);

                } else {
                    e.tick(&self.arena, dt, initial_tick);
                }

                // TODO send input to server
                self.remote_states.push((tick, e.get_state()));

                // Send all unconfirmed inputs to server
                let mut input_buffer = Vec::<u8>::new();
                e.serialize_inputs(&mut input_buffer);
                network.send(MessageKind::Instant, input_buffer);

                // TODO server side collision is checked on each server tick
                // positions are warped to the last known local tick of the player
                // BUT there is a maximum tick difference to prevent cheating

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

        // Reset entity ID map existing flag
        for e in self.entities.iter() {
            self.entity_id_map.insert(e.get_id(), false);
        }

        let mut i = 0;
        while i < data.len() {

            // TODO check number of bytes left

            // Entity ID

            // Entity Type

            // TODO create entites which are not yet in the map

            // Entity State

            // TODO if entity is local apply remote tick to entity
            // TODO otherwise simply apply state directly

            // TODO trigger create() method on entity
            // TODO trigger flag() method on entity if any flag changed

            i += 1;

        }

        // Collect missing IDs
        let removed_ids: Vec<u32> = self.entity_id_map
                                        .iter()
                                        .filter(|&(_, v)| *v == false)
                                        .map(|(&k, _)| k)
                                        .collect();

        // TODO destroy entities which are no longer in the map
        for id in removed_ids {
            // TODO trigger destroy() methods on entity
        }

    }

    pub fn draw(
        &mut self, core: &Core, prim: &PrimitivesAddon, font: &Font,
        network: &mut Network,
        dt: f32, u: f32
    ) {

        core.clear_to_color(self.back_color);

        // Draw all entities
        // TODO sort order?
        for e in self.entities.iter_mut() {
            e.draw(&core, &prim, &mut self.rng, &self.arena, dt, u);
        }

        // UI
        let network_state = match network.connected() {
            true => format!("Connected to server at {}", network.server_addr()),
            false => format!("Connecting to server at {}...", network.server_addr())
        };
        core.draw_text(font, self.text_color, 0.0, 0.0, FontAlign::Left, &network_state[..]);

    }

    fn reset(&mut self) {
        self.remote_states.clear();
        self.entity_id_map.clear();
        self.entities.clear();
    }

}

