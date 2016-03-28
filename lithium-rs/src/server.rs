// External Dependencies ------------------------------------------------------
use std::cmp;
use std::io::Error;
use std::net::SocketAddr;
use std::collections::BinaryHeap;
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
pub struct Server<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer, H: Handler<E, S, L, R>> {
    handler: H,
    manager: EntityManager<S, L, R>,
    events: EventHandler<E>,
    level: Level<S, L>,
    timer: Timer<E, S, L, R, H>
}

impl<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer, H: Handler<E, S, L, R>> Server<E, S, L, R, H> {

    // Statics ----------------------------------------------------------------
    pub fn run(addr: SocketAddr, mut server: Server<E, S, L, R, H>) -> Result<(), Error> where Self: Sized {

        let mut cobalt_server = CobaltServer::new(Config {
            send_rate: server.config().tick_rate as u32,
            .. Default::default()
        });

        cobalt_server.bind(&mut server, addr)

    }

    pub fn new(
        tick_rate: u32, buffer_ms: u32, interp_ms: u32,
        level: Level<S, L>,
        registry: Box<EntityRegistry<S, L, R>>,
        handler: H

    ) -> Server<E, S, L, R, H> {
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
    E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer, H: Handler<E, S, L, R>

> CobaltHandler<CobaltServer> for Server<E, S, L, R, H> {

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
    E: Event + 'a,
    S: EntityState + 'a,
    L: BaseLevel<S> + 'a,
    R: Renderer + 'a,
    H: Handler<E, S, L, R> + 'a
> {
    pub level: &'a mut Level<S, L>,
    pub entities: &'a mut EntityManager<S, L, R>,
    pub events: &'a mut EventHandler<E>,
    pub timer: &'a mut Timer<E, S, L, R, H>,
    pub server: &'a mut CobaltServer
}


// Server Handler -------------------------------------------------------------
pub trait Handler<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer> {

    fn bind(&mut self, Handle<E, S, L, R, Self>) where Self: Sized;
    fn connect(&mut self, Handle<E, S, L, R, Self>, &mut Connection) where Self: Sized;
    fn disconnect(&mut self, Handle<E, S, L, R, Self>, &mut Connection) where Self: Sized;

    fn event(&mut self, Handle<E, S, L, R, Self>, &mut ConnectionMap, ConnectionID, E) where Self: Sized;

    fn tick_before(&mut self, Handle<E, S, L, R, Self>, &mut ConnectionMap) where Self: Sized;

    fn tick_entity_before(&mut self, &Level<S, L>, &mut Entity<S, L, R>, u8, f32);
    fn tick_entity_after(&mut self, &Level<S, L>, &mut Entity<S, L, R>, u8, f32);

    fn tick_after(&mut self, Handle<E, S, L, R, Self>, &mut ConnectionMap) where Self: Sized;

    fn shutdown(&mut self, Handle<E, S, L, R, Self>) where Self: Sized;

}

// Timer Abstraction ----------------------------------------------------------
pub struct Timer<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer, H: Handler<E, S, L, R>> {
    callbacks: BinaryHeap<TimerCallback<E, S, L, R, H>>,
    time: u64,
    id: u32
}

impl<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer, H: Handler<E, S, L, R>> Timer<E, S, L, R, H> {

    pub fn new() -> Timer<E, S, L, R, H> {
        Timer {
            callbacks: BinaryHeap::new(),
            time: 0,
            id: 0
        }
    }

    pub fn update(&mut self, dt: u64) -> Vec<Box<FnMut(&mut H, Handle<E, S, L, R, H>)>> {

        self.time += dt;

        let mut callbacks = Vec::new();
        while {
            self.callbacks.peek().map_or(false, |c| {
                c.time <= self.time
            })
        } {
            // TODO check cancel list
            callbacks.push(self.callbacks.pop().unwrap().func);
        }

        callbacks

    }

    pub fn schedule(&mut self, f: Box<FnMut(&mut H, Handle<E, S, L, R, H>)>, time: u64) -> u32 {
        self.id += 1;
        self.callbacks.push(TimerCallback {
            func: f,
            time: self.time + time,
            id: self.id
        });
        self.id
    }

    pub fn cancel(&mut self, _: u32) {
        // TODO push into cancel list
    }

}


// Timer Callback Wrapper -----------------------------------------------------
struct TimerCallback<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer, H: Handler<E, S, L, R>> {
    func: Box<FnMut(&mut H, Handle<E, S, L, R, H>)>,
    time: u64,
    id: u32
}

impl<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer, H: Handler<E, S, L, R>> Eq for TimerCallback<E, S, L, R, H> {}

impl<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer, H: Handler<E, S, L, R>> PartialEq for TimerCallback<E, S, L, R, H> {
    fn eq(&self, other: &TimerCallback<E, S, L, R, H>) -> bool {
        self.id == other.id
    }
}

impl<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer, H: Handler<E, S, L, R>> Ord for TimerCallback<E, S, L, R, H> {
    // Explicitly implement the trait so the queue becomes a min-heap
    // instead of a max-heap.
    fn cmp(&self, other: &TimerCallback<E, S, L, R, H>) -> cmp::Ordering {
        other.time.cmp(&self.time)
    }
}

impl<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer, H: Handler<E, S, L, R>> PartialOrd for TimerCallback<E, S, L, R, H> {
    fn partial_cmp(&self, other: &TimerCallback<E, S, L, R, H>) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

