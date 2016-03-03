// External Dependencies ------------------------------------------------------
use std::net::SocketAddr;
use cobalt::{
    Config,
    Connection,
    ConnectionID,
    ConnectionMap,
    MessageKind,
    Handler as CobaltHandler,
    Server as CobaltServer
};


// Internal Dependencies ------------------------------------------------------
use network;
use entity::{
    Entity,
    EntityState,
    EntityInput,
    EntityManager,
    EntityManagerConfig,
    EntityRegistry
};
use level::{Level, BaseLevel};
use event::{Event, EventHandler};
use renderer::Renderer;


// Server Abstraction ---------------------------------------------------------
pub struct Server<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer> {
    handler: Box<Handler<E, S, L, R>>, // TODO unbox?
    manager: EntityManager<S, L, R>,
    events: EventHandler<E>,
    level: Level<S, L>
}

impl<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer> Server<E, S, L, R> {

    // Statics ----------------------------------------------------------------
    pub fn run(addr: SocketAddr, mut server: Server<E, S, L, R>) where Self: Sized {

        let mut cobalt_server = CobaltServer::new(Config {
            send_rate: server.config().tick_rate as u32,
            .. Config::default()
        });
        cobalt_server.bind(&mut server, addr).unwrap();

    }

    pub fn new(
        tick_rate: u32, buffer_ms: u32, interp_ms: u32,
        level: Level<S, L>,
        registry: Box<EntityRegistry<S, L, R>>,
        handler: Box<Handler<E, S, L, R>> // TODO unbox?

    ) -> Server<E, S, L, R> {
        Server {
            handler: handler,
            manager: EntityManager::new(
                tick_rate as u8, buffer_ms, interp_ms,
                true,
                registry
            ),
            events: EventHandler::new(),
            level: level
        }
    }


    // Public -----------------------------------------------------------------
    pub fn config(&self) -> &EntityManagerConfig {
        self.manager.config()
    }

}

impl<
    E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer

> CobaltHandler<CobaltServer> for Server<E, S, L, R> {

    fn bind(&mut self, _: &mut CobaltServer) {
        self.handler.bind(Handle {
            level: &mut self.level,
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
            level: &mut self.level,
            entities: &mut self.manager,
            events: &mut self.events

        }, conn);

    }

    fn tick_connections(
        &mut self, _: &mut CobaltServer, connections: &mut ConnectionMap
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
                            for i in data.chunks(EntityInput::encoded_size()) {
                                entity.remote_input(
                                    EntityInput::from_serialized(i)
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
                    level: &mut self.level,
                    entities: &mut self.manager,
                    events: &mut self.events

                }, owner, event);
            }
        }

        // Tick Entities
        let tick = self.manager.tick();

        {
            self.handler.tick_before(Handle {
                level: &mut self.level,
                entities: &mut self.manager,
                events: &mut self.events

            }, connections, tick, tick_dt);
        }

        self.manager.tick_server(&self.level, &mut self.handler, tick_dt);

        {
            self.handler.tick_after(Handle {
                level: &mut self.level,
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
            level: &mut self.level,
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
            level: &mut self.level,
            entities: &mut self.manager,
            events: &mut self.events
        });
    }

}


// Server Handle for Access from Handler ------------------------------------
pub struct Handle<
    'a, E: Event + 'a, S: EntityState + 'a, L: BaseLevel<S> + 'a, R: Renderer + 'a
> {
    pub level: &'a mut Level<S, L>,
    pub entities: &'a mut EntityManager<S, L, R>,
    pub events: &'a mut EventHandler<E>
}


// Server Handler -------------------------------------------------------------
pub trait Handler<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer> {

    fn bind(&mut self, Handle<E, S, L, R>);
    fn connect(&mut self, Handle<E, S, L, R>, &mut Connection);
    fn disconnect(&mut self, Handle<E, S, L, R>, &mut Connection);

    // TODO pass in connections mapping for all event / tick handles
    fn event(&mut self, Handle<E, S, L, R>, ConnectionID, E);

    fn tick_before(&mut self, Handle<E, S, L, R>, &mut ConnectionMap, u8, f32);

    fn tick_entity_before(&mut self, &Level<S, L>, &mut Entity<S, L, R>, u8, f32);
    fn tick_entity_after(&mut self, &Level<S, L>, &mut Entity<S, L, R>, u8, f32);

    fn tick_after(&mut self, Handle<E, S, L, R>, &mut ConnectionMap, u8, f32);

    fn shutdown(&mut self, Handle<E, S, L, R>);

}

