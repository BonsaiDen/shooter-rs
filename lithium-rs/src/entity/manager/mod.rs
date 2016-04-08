// External Dependencies ------------------------------------------------------
use std::collections::HashMap;
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
pub mod config;
pub mod registry;

use client;
use server;
use event::Event;
use idpool::IdPool;
use renderer::Renderer;
use level::{Level, BaseLevel};
use entity::{Entity, EntityState, EntityEvent};
use self::config::EntityManagerConfig;
use self::registry::EntityRegistry;


// Entity Manager Implementation ----------------------------------------------
pub struct EntityManager<S: EntityState, L: BaseLevel<S>, R: Renderer, G: EntityRegistry<S, L, R>> {

    // Id pool for entities
    id_pool: IdPool<u16>,

    // Vector of entities
    entities: HashMap<u16, Entity<S, L, R>>,

    // Configuration
    config: EntityManagerConfig,

    // Current tick
    tick: u8,

    // Wether to run in server mode
    server_mode: bool,

    // Entity Registry
    registry: G

}

impl<S: EntityState, L: BaseLevel<S>, R: Renderer, G: EntityRegistry<S, L, R>> EntityManager<S, L, R, G> {

    pub fn new(
        tick_rate: u8,
        buffer_ms: u32,
        interp_ms: u32,
        server_mode: bool,
        registry: G

    ) -> EntityManager<S, L, R, G> {
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

    pub fn dt(&self) -> f32 {
        1.0 / self.config.tick_rate as f32
    }

    pub fn config(&self) -> &EntityManagerConfig {
        &self.config
    }

    pub fn reset(&mut self) {

        let entity_ids: Vec<u16> = self.entities.keys().map(|id| *id).collect();
        for entity_id in entity_ids {
            self.destroy(entity_id);
        }

        assert_eq!(self.entities.len(), 0);
        self.entities.clear();

        assert_eq!(self.id_pool.len(), 0);
        self.id_pool.reset();
        self.tick = 0;

    }


    // Entities ---------------------------------------------------------------
    pub fn create(
        &mut self,
        type_id: u8,
        state: Option<S>,
        owner: Option<&ConnectionID>

    ) -> Option<&mut Entity<S, L, R>> {
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

            entity.event(EntityEvent::Created(self.tick, id));

            self.entities.insert(id, entity);
            self.entities.get_mut(&id)

        } else {
            None
        }
    }

    pub fn tick_server<E: Event, H: server::Handler<R, G, L, E, S>>(
        &mut self, level: &Level<S, L>, handler: &mut H
    ) {

        let dt = self.dt();
        for (_, entity) in &mut self.entities {
            handler.tick_entity_before(level, entity, self.tick, dt);
            entity.event(EntityEvent::Tick(self.tick, dt));
            entity.tick(level, self.tick, dt, self.server_mode);
            handler.tick_entity_after(level, entity, self.tick, dt);
        }

        self.tick = self.tick.wrapping_add(1);

    }

    pub fn tick_client<E: Event, H: client::Handler<R, G, L, E, S>>(
        &mut self,
        renderer: &mut R,
        level: &Level<S, L>,
        handler: &mut H

    ) -> Option<Vec<u8>> {

        let mut local_inputs: Option<Vec<u8>> = None;

        let dt = self.dt();
        for (_, entity) in &mut self.entities {

            handler.tick_entity_before(renderer, level, entity, self.tick, dt);
            entity.event(EntityEvent::Tick(self.tick, dt)); // TODO useful?
            entity.tick(level, self.tick, dt, self.server_mode);

            handler.tick_entity_after(renderer, level, entity, self.tick, dt);

            // Use serialized inputs from the locally controlled entities
            if let Some(inputs) = entity.serialized_inputs() {
                local_inputs = Some(inputs);
            }

        }

        self.tick = self.tick.wrapping_add(1);

        local_inputs

    }

    pub fn draw(&mut self, renderer: &mut R, level: &Level<S, L>) {
        for (_, entity) in &mut self.entities {
            if entity.is_visible() {
                entity.draw(renderer, level);
            }
        }
    }

    pub fn destroy(&mut self, entity_id: u16) -> Option<Entity<S, L, R>> {

        if let Some(mut entity) = self.entities.remove(&entity_id) {
            // TODO can be an issue if re-used directly
            self.id_pool.release_id(entity_id);
            entity.set_alive(false);
            entity.event(EntityEvent::Destroyed(self.tick, entity_id));
            Some(entity)

        } else {
            None
        }

    }

    pub fn get_entity_for_owner(
        &mut self, owner: &ConnectionID

    ) -> Option<&mut Entity<S, L, R>> {
        for (_, entity) in &mut self.entities {
            if entity.owned_by(owner) {
                return Some(entity);
            }
        }
        None
    }

    pub fn get_entity_id_for_owner(
        &mut self, owner: &ConnectionID

    ) -> Option<u16> {
        for (_, entity) in &mut self.entities {
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

    pub fn receive_config<'a>(&mut self, data: &'a [u8]) -> &'a [u8] {
        self.config = EntityManagerConfig::from_serialized(data);
        &data[EntityManagerConfig::encoded_size()..]
    }

    pub fn serialize_state(&self, owner: &ConnectionID) -> Vec<u8> {

        let mut state = Vec::new();

        // Serialize entity state for the connection
        for entity in self.entities.values() {
            state.extend(entity.serialize_state(owner));
        }

        state

    }

    pub fn receive_state(&mut self, data: &[u8]) {

        let tick = self.tick;
        let registry = &self.registry;
        let buffer_size = self.config.buffered_ticks as usize;

        // Mark all entities as dead
        for (_, entity) in &mut self.entities {
            entity.set_alive(false);
        }

        // Parse received state
        let mut i = 0;
        while i + Entity::<S, L, R>::header_size() <= data.len() {

            // Entity ID / Type
            let entity_id = (data[i] as u16) << 8 | (data[i + 1] as u16);
            let entity_type = data[i + 2];
            let entity_confirmed_tick = data[i + 3];
            let entity_is_visible = data[i + 4] == 1;
            i += Entity::<S, L, R>::header_size();

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

                entity.event(EntityEvent::Created(tick, entity_id));

                entity

            });

            // Handle entities which get hidden
            if entity.is_visible() {
                if !entity_is_visible {
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
        for (_, entity) in &mut self.entities {
            if !entity.alive() {
                let id = entity.id();
                entity.event(EntityEvent::Destroyed(tick, id));
                destroyed_ids.push(id);
            }
        }

        // ...then remove them from the map
        for id in &destroyed_ids {
            self.entities.remove(&id);
        }

    }


    // State Rewinding --------------------------------------------------------
    pub fn offset_states(&self, remote_tick: u8, rtt: f32) -> HashMap<u16, S> {

        // TODO add 

        // Calculate the server side state tick the client was actual displaying 
        // when they performed their action at "remote_tick".
        //
        // The client's server side state tick is delayed by both rtt / 2 and
        // the configured interpolation_ticks.
        let corrected_tick = remote_tick.wrapping_sub((
                // Ticks taken by ping
                (rtt / 2.0) / (1000.0 / self.config.tick_rate as f32)

            // Interpolation tick delay on the client
            ).ceil() as u8).wrapping_sub(self.config.interpolation_ticks);

        // Calculate tick offset into entity state buffers 
        let tick_offset = self.tick.wrapping_sub(corrected_tick) as usize;

        // Create a copy of all entity states at the given offset
        let mut states = HashMap::with_capacity(self.entities.len());
        for (id, entity) in &self.entities {
            states.insert(*id, entity.buffered_state(tick_offset).clone());
        }
        states

    }

}

