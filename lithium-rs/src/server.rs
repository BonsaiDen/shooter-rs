// External Dependencies ------------------------------------------------------
use std::cmp;
use std::io::Error;
use std::net::SocketAddr;
use std::collections::{BinaryHeap, HashSet};
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


// Macros ---------------------------------------------------------------------
macro_rules! handle {
    ($s:ident, $srv:ident) => {
       Handle {
            level: &mut $s.level,
            entities: &mut $s.manager,
            events: &mut $s.events,
            timer: &mut $s.timer,
            server: $srv
       }
    }
}


// Server Abstraction ---------------------------------------------------------
pub struct Server<
    H: Handler<R, G, L, E, S>, R: Renderer,
    G: EntityRegistry<S, L, R>, L: BaseLevel<S>, E: Event, S: EntityState
> {
    handler: H,
    manager: EntityManager<S, L, R, G>,
    events: EventHandler<E>,
    level: Level<S, L>,
    timer: Timer<H, R, G, L, E, S>
}

impl<
    H: Handler<R, G, L, E, S>, R: Renderer,
    G: EntityRegistry<S, L, R>, L: BaseLevel<S>, E: Event, S: EntityState

> Server<H, R, G, L, E, S> {

    // Statics ----------------------------------------------------------------
    pub fn run(
        addr: SocketAddr, mut server: Server<H, R, G, L, E, S>

    ) -> Result<(), Error> where Self: Sized {

        let mut cobalt_server = CobaltServer::new(Config {
            send_rate: server.config().tick_rate as u32,
            .. Default::default()
        });

        cobalt_server.bind(&mut server, addr)

    }

    pub fn new(
        tick_rate: u32, buffer_ms: u32, interp_ms: u32,
        level: Level<S, L>,
        registry: G,
        handler: H

    ) -> Server<H, R, G, L, E, S> {
        Server {
            handler: handler,
            manager: EntityManager::new(
                tick_rate as u8, buffer_ms, interp_ms,
                true,
                registry
            ),
            events: EventHandler::new(),
            level: level,
            timer: Timer::new()
        }
    }


    // Public -----------------------------------------------------------------
    pub fn config(&self) -> &EntityManagerConfig {
        self.manager.config()
    }

}

impl<
    H: Handler<R, G, L, E, S>, R: Renderer,
    G: EntityRegistry<S, L, R>, L: BaseLevel<S>, E: Event, S: EntityState

> CobaltHandler<CobaltServer> for Server<H, R, G, L, E, S> {

    fn bind(&mut self, server: &mut CobaltServer) {
        self.handler.bind(handle!(self, server));
    }

    fn connection(&mut self, server: &mut CobaltServer, conn: &mut Connection) {

        let mut config = [network::Message::ServerConfig as u8].to_vec();
        config.extend(self.manager.serialize_config());
        config.extend(self.level.serialize());
        conn.send(MessageKind::Reliable, config);

        self.handler.connect(handle!(self, server), conn);

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
                self.handler.event(handle!(self, server), connections, owner, event);
            }
        }

        // Tick Entities
        self.handler.tick_before(handle!(self, server), connections);
        self.manager.tick_server(&self.level, &mut self.handler);
        self.handler.tick_after(handle!(self, server), connections);

        // Run Timers
        let dt = self.manager.dt();
        let callbacks = self.timer.update((dt * 1000.0) as u64);
        for mut f in callbacks {
            f(&mut self.handler, handle!(self, server));
        }

        // Send Data
        for (id, conn) in connections.iter_mut() {

            // Send entity states to all clients (We don't care about dropped packets)
            let mut data = [network::Message::ServerState as u8].to_vec();
            data.extend(self.manager.serialize_state(id));
            conn.send(MessageKind::Instant, data);

            // Send events to all clients (Make sure the arrive eventually)
            // TODO event visibility handling
            if let Some(events) = self.events.serialize_events(Some(&id)) {
                let mut data = [network::Message::ServerEvents as u8].to_vec();
                data.extend(events);
                conn.send(MessageKind::Ordered, data);
            }

        }

        self.events.flush();

    }

    fn connection_lost(&mut self, server: &mut CobaltServer, conn: &mut Connection) {
        self.handler.disconnect(handle!(self, server), conn);
    }

    fn shutdown(&mut self, server: &mut CobaltServer) {
        self.handler.shutdown(handle!(self, server));
    }

}


// Server Handle for Access from Handler ------------------------------------
pub struct Handle<
    'a,
    H: Handler<R, G, L, E, S> + 'a,
    R: Renderer + 'a,
    G: EntityRegistry<S, L, R> + 'a,
    L: BaseLevel<S> + 'a,
    E: Event + 'a,
    S: EntityState + 'a
> {
    pub level: &'a mut Level<S, L>,
    pub entities: &'a mut EntityManager<S, L, R, G>,
    pub events: &'a mut EventHandler<E>,
    pub timer: &'a mut Timer<H, R, G, L, E, S>,
    pub server: &'a mut CobaltServer
}


// Server Handler -------------------------------------------------------------
pub trait Handler<
    R: Renderer,
    G: EntityRegistry<S, L, R>, L: BaseLevel<S>, E: Event, S: EntityState,
> {

    fn bind(&mut self, Handle<Self, R, G, L, E, S>) where Self: Sized;
    fn connect(&mut self, Handle<Self, R, G, L, E, S>, &mut Connection) where Self: Sized;
    fn disconnect(&mut self, Handle<Self, R, G, L, E, S>, &mut Connection) where Self: Sized;

    fn event(&mut self, Handle<Self, R, G, L, E, S>, &mut ConnectionMap, ConnectionID, E) where Self: Sized;

    fn tick_before(&mut self, Handle<Self, R, G, L, E, S>, &mut ConnectionMap) where Self: Sized;

    fn tick_entity_before(&mut self, &Level<S, L>, &mut Entity<S, L, R>, u8, f32);
    fn tick_entity_after(&mut self, &Level<S, L>, &mut Entity<S, L, R>, u8, f32);

    fn tick_after(&mut self, Handle<Self, R, G, L, E, S>, &mut ConnectionMap) where Self: Sized;

    fn shutdown(&mut self, Handle<Self, R, G, L, E, S>) where Self: Sized;

}


// Timer Implementation -------------------------------------------------------
impl_timer!(Handler, Renderer, EntityRegistry, BaseLevel, Event, EntityState);

