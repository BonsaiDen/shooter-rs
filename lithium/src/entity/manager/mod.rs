// External Dependencies ------------------------------------------------------
use std::collections::HashMap;
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
pub mod config;
pub mod registry;

use entity;
use level::Level;
use idpool::IdPool;
use renderer::Renderer;
use self::registry::EntityRegistry;
use self::config::EntityManagerConfig;


// Re-Exports -----------------------------------------------------------------
use self::config::EntityManagerConfig as Config;
use self::registry::EntityRegistry as Registry;



// Entity Manager Implementation ----------------------------------------------
pub struct EntityManager {

    // Id pool for entities
    id_pool: IdPool<u16>,

    // Vector of entities
    entities: HashMap<u16, entity::Entity>,

    // Configuration
    config: EntityManagerConfig,

    // Current tick
    tick: u8,

    // Wether to run in server mode
    server_mode: bool,

    // Entity Registry
    registry: Box<EntityRegistry>

}

impl EntityManager {

    pub fn new(
        tick_rate: u8,
        buffer_ms: u32,
        interp_ms: u32,
        server_mode: bool,
        registry: Box<EntityRegistry>

    ) -> EntityManager {
        EntityManager {
            id_pool: IdPool::new(),
            entities: HashMap::new(),
            config: EntityManagerConfig {
                buffered_ticks: (buffer_ms as f32 / (1000.0 / tick_rate as f32)).floor() as u8,
                interpolation_ticks: (interp_ms as f32 / (1000.0 / tick_rate as f32)).ceil() as u8,
                tick_rate: tick_rate,
            },
            tick: 0,
            server_mode: server_mode,
            registry: registry
        }
    }

    pub fn tick(&self) -> u8 {
        self.tick
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn init(&self, renderer: &mut Renderer) {
        renderer.set_tick_rate(self.config.tick_rate as u32);
        renderer.set_interpolation_ticks(
            self.config.interpolation_ticks as usize
        );
    }

    pub fn reset(&mut self) {
        self.tick = 0;
        self.entities.clear();
        self.id_pool.reset();
    }


    // Entities ---------------------------------------------------------------
    pub fn create_entity(
        &mut self,
        type_id: u8,
        state: Option<entity::State>,
        owner: Option<&ConnectionID>

    ) -> Option<&mut entity::Entity> {
        if let Some(id) = self.id_pool.get_id() {

            let mut entity = self.registry.entity_from_type_id(type_id);
            entity.set_buffer_size(self.config.buffered_ticks as usize);
            entity.set_id(id);
            entity.set_alive(true);

            if let Some(owner) = owner {
                entity.set_owner(*owner);
            }

            if let Some(state) = state {
                entity.set_state(state);
            }

            entity.event(entity::Event::Created(self.tick));

            self.entities.insert(id, entity);
            self.entities.get_mut(&id)

        } else {
            None
        }
    }

    pub fn tick_entities<B, A>(
        &mut self, level: &Level, dt: f32, mut before: B, mut after: A

    ) where B: FnMut(&mut entity::Entity, &Level, u8, f32),
            A: FnMut(&mut entity::Entity, &Level, u8, f32) {

        for (_, entity) in self.entities.iter_mut() {
            before(entity, level, self.tick, dt);
            entity.event(entity::Event::Tick(self.tick, dt)); // TODO useful?
            entity.tick(level, self.tick, dt, self.server_mode);
            after(entity, level, self.tick, dt);
        }

        if self.tick == 255 {
            self.tick = 0;

        } else {
            self.tick += 1;
        }

    }

    pub fn draw_entities(&mut self, renderer: &mut Renderer, level: &Level) {
        for (_, entity) in self.entities.iter_mut() {
            entity.draw(renderer, level);
        }
    }

    pub fn destroy_entity(&mut self, entity_id: u16) -> Option<entity::Entity> {

        if let Some(mut entity) = self.entities.remove(&entity_id) {
            // TODO can be an issue if re-used directly
            self.id_pool.release_id(entity.id());
            entity.set_alive(false);
            entity.event(entity::Event::Destroyed(self.tick));
            Some(entity)

        } else {
            None
        }

    }

    pub fn get_entity_for_owner(
        &mut self, owner: &ConnectionID

    ) -> Option<&mut entity::Entity> {
        for (_, entity) in self.entities.iter_mut() {
            if entity.owned_by(owner) {
                return Some(entity);
            }
        }
        None
    }

    pub fn get_entity_id_for_owner(
        &mut self, owner: &ConnectionID

    ) -> Option<u16> {
        for (_, entity) in self.entities.iter_mut() {
            if entity.owned_by(owner) {
                return Some(entity.id());
            }
        }
        None
    }


    // State Serialization ----------------------------------------------------
    pub fn serialize_config(&self) -> Vec<u8> {
        self.config.serialize()
    }

    pub fn receive_config<'a>(&mut self, renderer: &mut Renderer, data: &'a [u8]) -> &'a [u8] {
        self.config = EntityManagerConfig::from_serialized(data);
        renderer.set_interpolation_ticks(self.config.interpolation_ticks as usize);
        &data[EntityManagerConfig::encoded_size()..]
    }

    pub fn serialize_state(&self, owner: &ConnectionID) -> Vec<u8> {

        let mut state = Vec::new();
        for (_, entity) in self.entities.iter() {

            // TODO handle visibility with entity.visible_to(owner)
            // TODO still create entity but hide it and do not tranmit state?

            // Serialize entity state for the connection
            state.extend(entity.serialize_state(owner));

        }

        state

    }

    pub fn receive_state(&mut self, data: &[u8]) {

        let tick = self.tick;
        let registry = &self.registry;
        let buffer_size = self.config.buffered_ticks as usize;

        // Mark all entities as dead
        for (_, entity) in self.entities.iter_mut() {
            entity.set_alive(false);
        }

        // Parse received state
        let mut i = 0;
        while i + 4 <= data.len() {

            // Entity ID / Type
            let entity_id = (data[i] as u16) << 8 | (data[i + 1] as u16);
            let entity_type = data[i + 2];
            let entity_confirmed_tick = data[i + 3];
            i += 4;

            // Check serialized data length
            if i + entity::State::encoded_size() <= data.len() {

                // Read serialized entity state data
                let state = entity::State::from_serialized(&data[i..]);
                i += entity::State::encoded_size();

                // Create entities which do not yet exist
                let mut entity = self.entities.entry(entity_id).or_insert_with(|| {
                    let mut entity = registry.entity_from_type_id(entity_type);
                    entity.set_buffer_size(buffer_size);
                    entity.set_id(entity_id);
                    entity.set_state(state.clone());
                    entity.event(entity::Event::Created(tick));
                    entity
                });

                // Mark entity as alive
                entity.set_alive(true);

                // Set confirmed state...
                if entity.local() {
                    entity.set_confirmed_state(entity_confirmed_tick, state);

                // ...or overwrite local state
                // (but keep last_state intact for interpolation purposes)
                } else {
                    entity.set_remote_state(state);
                }

            }

        }

        // Destroy dead entities...
        let mut destroyed_ids = Vec::new();
        for (_, entity) in self.entities.iter_mut() {
            if entity.alive() == false {
                entity.event(entity::Event::Destroyed(tick));
                destroyed_ids.push(entity.id());
            }
        }

        // ...then remove them from the map
        for id in &destroyed_ids {
            self.entities.remove(&id);
        }

        // TODO networked events

    }

}

