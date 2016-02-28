// External Dependencies ------------------------------------------------------
use std::collections::HashMap;
use std::net::SocketAddr;
use cobalt::{
    Config,
    Connection,
    ConnectionID,
    MessageKind,
    Handler as CobaltHandler,
    Server as CobaltServer
};


// Internal Dependencies ------------------------------------------------------
use entity;
use event;
use level;
use network;


// Server Abstraction ---------------------------------------------------------
pub struct Server<E, L, S> where E: event::Event,
                                 L: level::Level<S>,
                                 S: entity::State
{
    handler: Box<Handler<E, L, S>>,
    manager: entity::Manager<S>,
    events: event::Handler<E>,
    level: L
}

impl<E, L, S> Server<E, L, S> where E: event::Event,
                                    L: level::Level<S>,
                                    S: entity::State

{

    // Statics ----------------------------------------------------------------
    pub fn run(
        addr: SocketAddr,
        mut server: Server<E, L, S>

    ) where Self: Sized {

        let mut cobalt_server = CobaltServer::new(Config {
            send_rate: server.config().tick_rate as u32,
            .. Config::default()
        });
        cobalt_server.bind(&mut server, addr).unwrap();

    }

    pub fn new(
        tick_rate: u32, buffer_ms: u32, interp_ms: u32,
        level: L,
        registry: Box<entity::Registry<S>>,
        handler: Box<Handler<E, L, S>>

    ) -> Server<E, L, S> {
        Server {
            handler: handler,
            manager: entity::Manager::new(
                tick_rate as u8, buffer_ms, interp_ms,
                true,
                registry
            ),
            events: event::Handler::new(),
            level: level
        }
    }


    // Public -----------------------------------------------------------------
    pub fn config(&self) -> &entity::ManagerConfig {
        self.manager.config()
    }

}

impl<E, L, S> CobaltHandler<CobaltServer> for Server<E, L, S>
where E: event::Event, L: level::Level<S>, S: entity::State {

    fn bind(&mut self, _: &mut CobaltServer) {
        self.handler.bind(Handle {
            level: &self.level,
            entities: &mut self.manager,
            events: &mut self.events
        });
    }

    fn connection(&mut self, _: &mut CobaltServer, conn: &mut Connection) {

        let mut config = [network::Message::ServerConfig as u8].to_vec();
        config.extend(self.manager.serialize_config());
        config.extend(self.level.serialize());
        conn.send(MessageKind::Reliable, config);

        self.handler.connect(Handle {
            level: &self.level,
            entities: &mut self.manager,
            events: &mut self.events

        }, conn);

    }

    fn tick_connections(
        &mut self, _: &mut CobaltServer,
        connections: &mut HashMap<ConnectionID, Connection>
    ) {

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

        // Handle events
        if let Some(events) = self.events.received() {
            for (owner, event) in events {
                self.handler.event(Handle {
                    level: &self.level,
                    entities: &mut self.manager,
                    events: &mut self.events

                }, owner, event);
            }
        }

        // Tick Entities
        let tick = self.manager.tick();

        {
            self.handler.tick_before(Handle {
                level: &self.level,
                entities: &mut self.manager,
                events: &mut self.events

            }, connections, tick, tick_dt);
        }

        self.manager.tick_server(&self.level, &mut self.handler, tick_dt);

        {
            self.handler.tick_after(Handle {
                level: &self.level,
                entities: &mut self.manager,
                events: &mut self.events

            }, connections, tick, tick_dt);
        }

        // Send Data
        for (id, conn) in connections.iter_mut() {

            // Send entity states to all clients (We don't care about dropped packets)
            let mut data = [network::Message::ServerState as u8].to_vec();
            data.extend(self.manager.serialize_state(id));
            conn.send(MessageKind::Instant, data);

            // Send events to all clients (Make sure the arrive eventually)
            // TODO potential issues with events for entities which
            // do not yet exist or have already been destroyed
            // fix: delay events and drop them eventually (after some specified time)?
            if let Some(ref events) = self.events.serialize_events(Some(&id)) {
                // TODO filter events here?
                let mut data = [network::Message::ServerEvents as u8].to_vec();
                data.extend(events.clone());
                conn.send(MessageKind::Reliable, data);
            }

        }

        self.events.flush();

    }

    fn connection_lost(&mut self, _: &mut CobaltServer, conn: &mut Connection) {
        self.handler.disconnect(Handle {
            level: &self.level,
            entities: &mut self.manager,
            events: &mut self.events

        }, conn);
    }

    fn connection_packet_lost(
        &mut self, _: &mut CobaltServer, _: &mut Connection, _: &[u8]
    ) {
    }

    fn connection_congestion_state(
        &mut self, _: &mut CobaltServer, _: &mut Connection, _: bool
    ) {
    }

    fn shutdown(&mut self, _: &mut CobaltServer) {
        self.handler.shutdown(Handle {
            level: &self.level,
            entities: &mut self.manager,
            events: &mut self.events
        });
    }

}


// Server Handle for Access from Handler ------------------------------------
pub struct Handle<
    'a,
    E: event::Event + 'a,
    L: level::Level<S> + 'a,
    S: entity::State + 'a
> {
    pub level: &'a L,
    pub entities: &'a mut entity::Manager<S>,
    pub events: &'a mut event::Handler<E>
}


// Server Handler -------------------------------------------------------------
pub trait Handler<E: event::Event, L: level::Level<S>, S: entity::State> {

    fn bind(&mut self, Handle<E, L, S>);
    fn connect(&mut self, Handle<E, L, S>, &mut Connection);
    fn disconnect(&mut self, Handle<E, L, S>, &mut Connection);

    // TODO pass in connections mapping for all event / tick handles
    fn event(&mut self, Handle<E, L, S>, ConnectionID, E);

    fn tick_before(
        &mut self,
        Handle<E, L, S>,
        &mut HashMap<ConnectionID, Connection>,
        u8, f32
    );

    fn tick_entity_before(&mut self, &L, &mut entity::Entity<S>, u8, f32);
    fn tick_entity_after(&mut self, &L, &mut entity::Entity<S>, u8, f32);

    fn tick_after(
        &mut self,
        Handle<E, L, S>,
        &mut HashMap<ConnectionID, Connection>,
        u8, f32
    );

    fn shutdown(&mut self, Handle<E, L, S>);

}

