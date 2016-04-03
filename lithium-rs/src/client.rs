// External Dependencies ------------------------------------------------------
use std::cmp;
use std::collections::{BinaryHeap, HashSet};
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
pub struct Client<
    H: Handler<R, G, L, E, S>, R: Renderer,
    G: EntityRegistry<S, L, R>, L: BaseLevel<S>, E: Event, S: EntityState
> {
    handler: H,
    client: ClientStream,
    manager: EntityManager<S, L, R, G>,
    events: EventHandler<E>,
    level: Level<S, L>,
    timer: Timer<H, R, G, L, E, S>
}

impl<
    H: Handler<R, G, L, E, S>, R: Renderer,
    G: EntityRegistry<S, L, R>, L: BaseLevel<S>, E: Event, S: EntityState

> Client<H, R, G, L, E, S> {

    // Statics ----------------------------------------------------------------
    pub fn new(
        tick_rate: u8,
        level: Level<S, L>,
        registry: G,
        handler: H

    ) -> Client<H, R, G, L, E, S> {
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

                ClientEvent::ConnectionLost => {
                    self.manager.reset();
                    self.handler.disconnect(handle!(self, renderer), true, false);
                    self.client.close().ok();
                },

                ClientEvent::ConnectionClosed(by_remote) => {
                    self.manager.reset();
                    self.handler.disconnect(handle!(self, renderer), true, by_remote);
                    self.client.close().ok();
                },

                ClientEvent::ConnectionFailed => {
                    self.handler.disconnect(handle!(self, renderer), false, false);
                    //self.network.close().ok(); // TODO this screws up reconnect logic
                },

                ClientEvent::Close => {
                    println!("Close event");
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
    H: Handler<R, G, L, E, S> + 'a,
    R: Renderer + 'a,
    G: EntityRegistry<S, L, R> + 'a,
    L: BaseLevel<S> + 'a,
    E: Event + 'a,
    S: EntityState + 'a
> {
    pub renderer: &'a mut R,
    pub level: &'a mut Level<S, L>,
    pub events: &'a mut EventHandler<E>,
    pub entities: &'a mut EntityManager<S, L, R, G>,
    pub timer: &'a mut Timer<H, R, G, L, E, S>,
    pub client: &'a mut ClientStream
}


// Client Handler -------------------------------------------------------------
pub trait Handler<
    R: Renderer,
    G: EntityRegistry<S, L, R>, L: BaseLevel<S>, E: Event, S: EntityState,
> {

    fn init(&mut self, Handle<Self, R, G, L, E, S>) where Self: Sized;
    fn connect(&mut self, Handle<Self, R, G, L, E, S>) where Self: Sized;
    fn disconnect(&mut self, Handle<Self, R, G, L, E, S>, bool, bool) where Self: Sized;

    fn config(&mut self, Handle<Self, R, G, L, E, S>, &[u8]) where Self: Sized;

    fn event(&mut self, Handle<Self, R, G, L, E, S>, ConnectionID, E) where Self: Sized;

    fn tick_before(&mut self, Handle<Self, R, G, L, E, S>) where Self: Sized;

    fn tick_entity_before(
        &mut self, &mut R, &Level<S, L>, &mut Entity<S, L, R>, u8, f32
    );

    fn tick_entity_after(
        &mut self, &mut R, &Level<S, L>, &mut Entity<S, L, R>, u8, f32
    );

    fn tick_after(&mut self, Handle<Self, R, G, L, E, S>) where Self: Sized;

    fn draw(&mut self, Handle<Self, R, G, L, E, S>) where Self: Sized;

    fn destroy(&mut self, Handle<Self, R, G, L, E, S>) where Self: Sized;

}


// Timer Implementation -------------------------------------------------------
impl_timer!(Handler, Renderer, EntityRegistry, BaseLevel, Event, EntityState);

