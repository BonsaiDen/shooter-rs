// External Dependencies ------------------------------------------------------
use std::cmp;
use std::collections::BinaryHeap;
use cobalt::{Config, ConnectionID, ClientStream, ClientEvent, MessageKind};


// Internal Dependencies ------------------------------------------------------
use network;
use entity::{
    Entity,
    EntityState,
    EntityManager,
    EntityRegistry
};
use level::{Level, BaseLevel};
use event::{Event, EventHandler};
use renderer::Renderer;


// Macros ---------------------------------------------------------------------
macro_rules! handle {
    ($s:ident, $r:ident) => {
       Handle {
            renderer: $r,
            level: &mut $s.level,
            events: &mut $s.events,
            entities: &mut $s.manager,
            timer: &mut $s.timer,
            client: &mut $s.client
       }
    }
}


// Client Abstraction ---------------------------------------------------------
pub struct Client<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer, H: Handler<E, S, L, R>> {
    handler: H,
    client: ClientStream,
    manager: EntityManager<S, L, R>,
    events: EventHandler<E>,
    level: Level<S, L>,
    timer: Timer<E, S, L, R, H>
}

impl<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer, H: Handler<E, S, L, R>> Client<E, S, L, R, H> {

    // Statics ----------------------------------------------------------------
    pub fn new(
        tick_rate: u8,
        level: Level<S, L>,
        registry: Box<EntityRegistry<S, L, R>>,
        handler: H

    ) -> Client<E, S, L, R, H> {
        Client {
            handler: handler,
            client: ClientStream::new(Config {
                send_rate: tick_rate as u32,
                connection_init_threshold: 250,
                .. Default::default()
            }),
            manager: EntityManager::new(
                tick_rate, 1000, 75,
                false,
                registry
            ),
            events: EventHandler::new(),
            level: level,
            timer: Timer::new()
        }
    }


    // Public -----------------------------------------------------------------
    pub fn init(&mut self, renderer: &mut R) {
        self.update_tick_config(renderer);
        self.handler.init(handle!(self, renderer));
    }

    pub fn destroy(&mut self, renderer: &mut R) {

        self.handler.destroy(handle!(self, renderer));

        // Send any pending outgoing events
        self.send_events();
        self.client.flush().ok();
        self.client.close().ok();

    }


    // Tick Handling ----------------------------------------------------------
    pub fn tick(&mut self, renderer: &mut R) -> bool {

        let mut ticked = false;

        while let Ok(event) = self.client.receive() {
            match event {

                ClientEvent::Connection => {
                    self.manager.reset();
                    self.handler.connect(handle!(self, renderer));
                },

                ClientEvent::Message(data) =>  {
                    match network::Message::from_u8(data[0]) {

                        network::Message::ServerConfig => {
                            let data = self.manager.receive_config(&data[1..]);
                            self.update_tick_config(renderer);
                            self.handler.config(handle!(self, renderer), data);
                        },

                        network::Message::ServerState => {
                            self.manager.receive_state(&data[1..]);
                        },

                        network::Message::ServerEvents => {
                            self.events.receive_events(
                                ConnectionID(0),
                                &data[1..]
                            );
                        },

                        _=> println!("Unknown Server Message {:?}", data)

                    }
                },

                ClientEvent::Tick => {

                    if let Some(events) = self.events.received() {
                        for (owner, event) in events {
                            self.handler.event(handle!(self, renderer), owner, event);
                        }
                    }

                    self.handler.tick_before(handle!(self, renderer));

                    self.tick_entities(renderer);
                    self.send_events();

                    self.handler.tick_after(handle!(self, renderer));

                    ticked = true;

                },

                ClientEvent::Close | ClientEvent::ConnectionLost => {
                    self.manager.reset();
                    self.handler.disconnect(handle!(self, renderer), true);
                    self.client.close().ok();
                },

                ClientEvent::ConnectionClosed(by_remote) => {
                    self.manager.reset();
                    // TODO by_remote should be there along with was_connected?
                    self.handler.disconnect(handle!(self, renderer), by_remote);
                    self.client.close().ok();
                },

                ClientEvent::ConnectionFailed => {
                    self.handler.disconnect(handle!(self, renderer), false);
                    //self.network.close().ok(); // TODO this screws up reconnect logic
                },

                _ => {}

            }
        }

        self.client.flush().ok();

        ticked

    }

    pub fn draw(&mut self, renderer: &mut R) {

        // Run Timers
        let dt = renderer.delta_time();
        let callbacks = self.timer.update((dt * 1000.0) as u64);
        for mut f in callbacks {
            f(&mut self.handler, handle!(self, renderer));
        }

        self.handler.draw(handle!(self, renderer));

    }


    // Internal ---------------------------------------------------------------
    fn update_tick_config(&mut self, renderer: &mut R) {
        let tick_rate = self.manager.config().tick_rate as u32;
        let config = self.client.config();
        self.client.set_config(Config {
            send_rate: tick_rate,
            .. config
        });
        renderer.set_tick_rate(tick_rate);
        renderer.set_interpolation_ticks(
            self.manager.config().interpolation_ticks as usize
        );
    }

    fn tick_entities(&mut self, renderer: &mut R) {

        let local_inputs = self.manager.tick_client(
            renderer, &self.level, &mut self.handler
        );

        if let Some(inputs) = local_inputs {
            self.send_message(
                MessageKind::Instant,
                network::Message::ClientInput,
                &inputs
            );
        }

    }

    fn send_events(&mut self) {

        if let Some(events) = self.events.serialize_events(None) {
            self.send_message(
                MessageKind::Ordered,
                network::Message::ClientEvents,
                &events
            );
        }

        self.events.flush();

    }

    fn send_message(&mut self, kind: MessageKind, typ: network::Message, data: &[u8]) {
        let mut msg = [typ as u8].to_vec();
        msg.extend_from_slice(data);
        self.client.send(kind, msg).ok();
    }

}


// Client Handle for Access from Handler --------------------------------------
pub struct Handle<
    'a,
    E: Event + 'a,
    S: EntityState + 'a,
    L: BaseLevel<S> + 'a,
    R: Renderer + 'a,
    H: Handler<E, S, L, R> + 'a
> {
    pub renderer: &'a mut R,
    pub level: &'a mut Level<S, L>,
    pub events: &'a mut EventHandler<E>,
    pub entities: &'a mut EntityManager<S, L, R>,
    pub timer: &'a mut Timer<E, S, L, R, H>,
    pub client: &'a mut ClientStream
}


// Client Handler -------------------------------------------------------------
pub trait Handler<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer> {

    fn init(&mut self, Handle<E, S, L, R, Self>) where Self: Sized;
    fn connect(&mut self, Handle<E, S, L, R, Self>) where Self: Sized;
    fn disconnect(&mut self, Handle<E, S, L, R, Self>, bool) where Self: Sized;

    fn config(&mut self, Handle<E, S, L, R, Self>, &[u8]) where Self: Sized;

    fn event(&mut self, Handle<E, S, L, R, Self>, ConnectionID, E) where Self: Sized;

    fn tick_before(&mut self, Handle<E, S, L, R, Self>) where Self: Sized;

    fn tick_entity_before(
        &mut self, &mut R, &Level<S, L>, &mut Entity<S, L, R>, u8, f32
    );

    fn tick_entity_after(
        &mut self, &mut R, &Level<S, L>, &mut Entity<S, L, R>, u8, f32
    );

    fn tick_after(&mut self, Handle<E, S, L, R, Self>) where Self: Sized;

    fn draw(&mut self, Handle<E, S, L, R, Self>) where Self: Sized;

    fn destroy(&mut self, Handle<E, S, L, R, Self>) where Self: Sized;

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


