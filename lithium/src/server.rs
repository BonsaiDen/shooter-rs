// External Dependencies ------------------------------------------------------
use std::collections::HashMap;
use cobalt::{Connection, ConnectionID, MessageKind};


// Internal Dependencies ------------------------------------------------------
use entity;
use event;
use level;
use network;


// Server Abstraction ---------------------------------------------------------
pub struct Server<E, L> where E: event::Event, L: level::Level {
    manager: entity::Manager,
    events: event::Handler<E>,
    level: L
}

impl<E, L> Server<E, L> where E: event::Event, L: level::Level {

    pub fn new(
        tick_rate: u32, buffer_ms: u32, interp_ms: u32,
        level: L, registry: Box<entity::Registry>

    ) -> Server<E, L> {
        Server {
            manager: entity::Manager::new(
                tick_rate as u8, buffer_ms, interp_ms,
                true,
                registry
            ),
            events: event::Handler::new(),
            level: level
        }
    }

    pub fn tick<B, A>(
        &mut self,
        connections: &mut HashMap<ConnectionID, Connection>,
        before: B, after: A

    ) where B: FnMut(&mut entity::Entity, &level::Level, u8, f32),
            A: FnMut(&mut entity::Entity, &level::Level, u8, f32) {

        let tick_dt = 1.0 / self.manager.config().tick_rate as f32;

        // Receive Data
        for (id, conn) in connections.iter_mut() {
            for data in conn.received() {
                match network::Message::from_u8(data[0]) {
                    network::Message::ClientInput => {

                        // Extract all unconfirmed inputs the client sent us
                        if let Some(entity) = self.manager.get_entity_for_owner(id) {
                            let data = &data[1..];
                            for i in data.chunks(entity::Input::encoded_size()) {
                                entity.remote_input(
                                    entity::Input::from_serialized(i)
                                );
                            }
                        }

                    },
                    network::Message::ClientEvents => {
                        self.events.receive_events(*id, &data[1..]);
                    },
                    _=> println!("Unknown Client Message {:?}", data)
                }
            }
        }

        // Tick Entities
        self.manager.tick_server_entities(&self.level, tick_dt, before, after);

        // Send Data
        let events = self.events.serialize_events();
        for (id, conn) in connections.iter_mut() {

            // Send entity states to all clients (We don't care about dropped packets)
            let mut data = [network::Message::ServerState as u8].to_vec();
            data.extend(self.manager.serialize_state(id));
            conn.send(MessageKind::Instant, data);

            // Send events to all clients (Make sure the arrive eventually)
            // TODO potential issues with events for entities when entities
            // do not yet exist or have already been destroyed
            // delay events and drop them eventually (after some specified time)?
            if let Some(ref events) = events {
                let mut data = [network::Message::ServerEvents as u8].to_vec();
                data.extend(events.clone());
                conn.send(MessageKind::Reliable, data);
            }

        }

    }


    // Connection Interface ---------------------------------------------------
    pub fn init_connection(&self, conn: &mut Connection) {
        let mut config = [network::Message::ServerConfig as u8].to_vec();
        config.extend(self.manager.serialize_config());
        config.extend(self.level.serialize());
        conn.send(MessageKind::Reliable, config);
    }

    pub fn close_connection<N>(
        &mut self,
        conn: &mut Connection,
        mut destroy_entity: N

    ) where N: FnMut(&mut Connection, entity::Entity) {
        while let Some(id) = self.manager.get_entity_id_for_owner(&conn.id()) {
            if let Some(entity) = self.manager.destroy_entity(id) {
                destroy_entity(conn, entity);
            }
        }
    }


    // Level Interface --------------------------------------------------------
    pub fn level(&mut self) -> &mut L {
        &mut self.level
    }


    // Entity Interface -------------------------------------------------------
    pub fn entities(&mut self) -> &mut entity::Manager {
        &mut self.manager
    }


    // Event Interface --------------------------------------------------------
    pub fn events(&mut self) -> &mut event::Handler<E> {
        &mut self.events
    }

    // TODO support entity events?
    // TODO send event? or do this via state updates only?
    // probably send player joined event but add entity via
    // state change detection on client

}

