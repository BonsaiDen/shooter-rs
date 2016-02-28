// External Dependencies ------------------------------------------------------
use std::cmp;
use std::collections::HashMap;
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
pub mod config;
pub mod registry;

use entity;
use client;
use server;
use event::Event;
use level::Level;
use idpool::IdPool;
use renderer::Renderer;
use self::registry::EntityRegistry;
use self::config::EntityManagerConfig;


// Re-Exports -----------------------------------------------------------------
use self::config::EntityManagerConfig as Config;
use self::registry::EntityRegistry as Registry;


// Entity Manager Implementation ----------------------------------------------
pub struct EntityManager<S: entity::State> {

    // Id pool for entities
    id_pool: IdPool<u16>,

    // Vector of entities
    entities: HashMap<u16, entity::Entity<S>>,

    // Configuration
    config: EntityManagerConfig,

    // Current tick
    tick: u8,

    // Wether to run in server mode
    server_mode: bool,

    // Entity Registry
    registry: Box<EntityRegistry<S>>

}

impl<S> EntityManager<S> where S: entity::State {

    pub fn new(
        tick_rate: u8,
        buffer_ms: u32,
        interp_ms: u32,
        server_mode: bool,
        registry: Box<EntityRegistry<S>>

    ) -> EntityManager<S> {
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
    pub fn create(
        &mut self,
        type_id: u8,
        state: Option<S>,
        owner: Option<&ConnectionID>

    ) -> Option<&mut entity::Entity<S>> {
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

    pub fn tick_server<E, L>(
        &mut self,
        level: &L,
        handler: &mut Box<server::Handler<E, L, S>>,
        dt: f32

    ) where E: Event, L: Level<S> {

        for (_, entity) in self.entities.iter_mut() {
            handler.tick_entity_before(level, entity, self.tick, dt);
            entity.event(entity::Event::Tick(self.tick, dt)); // TODO useful?
            entity.tick(level, self.tick, dt, self.server_mode);
            handler.tick_entity_after(level, entity, self.tick, dt);
        }

        if self.tick == 255 {
            self.tick = 0;

        } else {
            self.tick += 1;
        }

    }

    pub fn tick_client<E, L, I>(
        &mut self,
        renderer: &mut Renderer,
        handler: &mut client::Handler<E, L, S>,
        level: &L, dt: f32,
        mut input_handler: I

    ) where E: Event,
            L: Level<S>,
            I: FnMut(entity::ControlState, &mut entity::Entity<S>, u8)
    {
        for (_, entity) in self.entities.iter_mut() {
            handler.tick_entity_before(renderer, level, entity, self.tick, dt);
            entity.event(entity::Event::Tick(self.tick, dt)); // TODO useful?
            entity.tick(level, self.tick, dt, self.server_mode);
            match handler.tick_entity_after(renderer, level, entity, self.tick, dt) {
                entity::ControlState::None => {},
                state => input_handler(state, entity, self.tick)
            }
        }

        if self.tick == 255 {
            self.tick = 0;

        } else {
            self.tick += 1;
        }

    }


    pub fn draw(&mut self, renderer: &mut Renderer, level: &Level<S>) {
        for (_, entity) in self.entities.iter_mut() {
            if entity.is_visible() {
                entity.draw(renderer, level);
            }
        }
    }

    pub fn destroy(&mut self, entity_id: u16) -> Option<entity::Entity<S>> {

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

    ) -> Option<&mut entity::Entity<S>> {
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

        // Serialize entity state for the connection
        for (_, entity) in self.entities.iter() {
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
        while i + entity::Entity::<S>::header_size() <= data.len() {

            // Entity ID / Type
            let entity_id = (data[i] as u16) << 8 | (data[i + 1] as u16);
            let entity_type = data[i + 2];
            let entity_confirmed_tick = data[i + 3];
            let entity_is_visible = data[i + 4] == 1;
            i += entity::Entity::<S>::header_size();

            // Read serialized entity state data for visible entities
            let entity_state = if entity_is_visible && S::encoded_size() <= data.len() {
                let state = S::from_serialized(&data[i..]);
                i += S::encoded_size();
                Some(state)

            } else {
                None
            };

            // Create entities which do not yet exist
            let mut entity = self.entities.entry(entity_id).or_insert_with(|| {

                let mut entity = registry.entity_from_type_id(entity_type);
                entity.set_buffer_size(buffer_size);
                entity.set_id(entity_id);

                // Set state if we got any
                if let Some(ref state) = entity_state {
                    entity.set_state(state.clone());
                    entity.show(tick);

                } else {
                    entity.hide(tick);
                }

                entity.event(entity::Event::Created(tick));

                entity

            });

            // Handle entities which get hidden
            if entity.is_visible() {
                if entity_is_visible == false {
                    entity.hide(tick);
                }

            // Handle entities which get display (only if we got state though)
            } else if entity_is_visible {
                if let Some(ref state) = entity_state {
                    entity.set_state(state.clone());
                    entity.show(tick);
                }
            }

            // Mark entity as alive
            entity.set_alive(true);

            // Set confirmed state if we got any...
            if let Some(state) = entity_state {
                if entity.local() {
                    entity.set_confirmed_state(entity_confirmed_tick, state);

                // ...or overwrite local state
                } else {
                    // But keep last_state intact for interpolation purposes
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

    }


    // State Rewinding --------------------------------------------------------
    pub fn rewind<'a>(&'a mut self, tick: u8) -> StateRewinder<'a, S> {

        let tick_offset = cmp::max(0, self.tick - tick) as usize;
        for (_, entity) in self.entities.iter_mut() {
            entity.rewind_state(tick_offset);
        }

        StateRewinder {
            manager: self
        }

    }

    fn forward(&mut self) {
        for (_, entity) in self.entities.iter_mut() {
            entity.forward_state();
        }
    }

}


// Handle for rewinded entity state -------------------------------------------
pub struct StateRewinder<'a, S: entity::State + 'a> {
    manager: &'a mut EntityManager<S>
}

impl<'a, S> Drop for StateRewinder<'a, S> where S: entity::State {
    fn drop(&mut self) {
        self.manager.forward();
    }
}

