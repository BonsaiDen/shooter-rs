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
            .. Default::default()
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

    fn bind(&mut self, server: &mut CobaltServer) {
        self.handler.bind(Handle {
            level: &mut self.level,
            entities: &mut self.manager,
            events: &mut self.events,
            server: server
        });
    }

    fn connection(&mut self, server: &mut CobaltServer, conn: &mut Connection) {

        let mut config = [network::Message::ServerConfig as u8].to_vec();
        config.extend(self.manager.serialize_config());
        config.extend(self.level.serialize());
        conn.send(MessageKind::Reliable, config);

        self.handler.connect(Handle {
            level: &mut self.level,
            entities: &mut self.manager,
            events: &mut self.events,
            server: server

        }, conn);

    }

    fn tick_connections(
        &mut self, server: &mut CobaltServer, connections: &mut ConnectionMap
    ) {

        // Receive Data
        for (id, conn) in connections.iter_mut() {
            for msg in conn.received() {
                match network::Message::from_u8(msg[0]) {

                    network::Message::ClientInput => {
                        // Extract all unconfirmed inputs the client sent us
                        if let Some(entity) = self.manager.get_entity_for_owner(id) {
                            let msg = &msg[1..];
                            for i in msg.chunks(EntityInput::encoded_size()) {
                                entity.remote_input(
                                    EntityInput::from_serialized(i)
                                );
                            }
                        }
                    },

                    network::Message::ClientEvents => {
                        self.events.receive_events(*id, &msg[1..]);
                    },

                    _=> println!("Unknown Client Message {:?}", msg)

                }
            }
        }

        // Handle events
        if let Some(events) = self.events.received() {
            for (owner, event) in events {
                self.handler.event(Handle {
                    level: &mut self.level,
                    entities: &mut self.manager,
                    events: &mut self.events,
                    server: server

                }, connections, owner, event);
            }
        }

        // Tick Entities
        self.handler.tick_before(Handle {
            level: &mut self.level,
            entities: &mut self.manager,
            events: &mut self.events,
            server: server

        }, connections);

        self.manager.tick_server(&self.level, &mut self.handler);

        self.handler.tick_after(Handle {
            level: &mut self.level,
            entities: &mut self.manager,
            events: &mut self.events,
            server: server

        }, connections);

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
            if let Some(events) = self.events.serialize_events(Some(&id)) {
                let mut data = [network::Message::ServerEvents as u8].to_vec();
                data.extend(events);
                conn.send(MessageKind::Ordered, data);
            }

        }

        self.events.flush();

    }

    fn connection_lost(&mut self, server: &mut CobaltServer, conn: &mut Connection) {
        self.handler.disconnect(Handle {
            level: &mut self.level,
            entities: &mut self.manager,
            events: &mut self.events,
            server: server

        }, conn);
    }

    fn shutdown(&mut self, server: &mut CobaltServer) {
        self.handler.shutdown(Handle {
            level: &mut self.level,
            entities: &mut self.manager,
            events: &mut self.events,
            server: server
        });
    }

}


// Server Handle for Access from Handler ------------------------------------
pub struct Handle<
    'a,
    E: Event + 'a,
    S: EntityState + 'a,
    L: BaseLevel<S> + 'a,
    R: Renderer + 'a
> {
    pub level: &'a mut Level<S, L>,
    pub entities: &'a mut EntityManager<S, L, R>,
    pub events: &'a mut EventHandler<E>,
    pub server: &'a mut CobaltServer
}


// Server Handler -------------------------------------------------------------
pub trait Handler<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer> {

    fn bind(&mut self, Handle<E, S, L, R>);
    fn connect(&mut self, Handle<E, S, L, R>, &mut Connection);
    fn disconnect(&mut self, Handle<E, S, L, R>, &mut Connection);

    fn event(&mut self, Handle<E, S, L, R>, &mut ConnectionMap, ConnectionID, E);

    fn tick_before(&mut self, Handle<E, S, L, R>, &mut ConnectionMap);

    fn tick_entity_before(&mut self, &Level<S, L>, &mut Entity<S, L, R>, u8, f32);
    fn tick_entity_after(&mut self, &Level<S, L>, &mut Entity<S, L, R>, u8, f32);

    fn tick_after(&mut self, Handle<E, S, L, R>, &mut ConnectionMap);

    fn shutdown(&mut self, Handle<E, S, L, R>);

}

