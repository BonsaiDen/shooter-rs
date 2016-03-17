// External Dependencies ------------------------------------------------------
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


// Client Abstraction ---------------------------------------------------------
pub struct Client<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer> {
    network: ClientStream, // TODO rename to stream?
    manager: EntityManager<S, L, R>,
    events: EventHandler<E>,
    remote_states: Vec<(u8, S)>,
    level: Level<S, L>
}

impl<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer> Client<E, S, L, R> {

    // Statics ----------------------------------------------------------------
    pub fn new(
        tick_rate: u8,
        level: Level<S, L>,
        registry: Box<EntityRegistry<S, L, R>>

    ) -> Client<E, S, L, R> {
        Client {
            network: ClientStream::new(Config {
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
            remote_states: Vec::new(),
            level: level
        }
    }


    // Public -----------------------------------------------------------------
    pub fn init<H: Handler<E, S, L, R>>(
        &mut self, handler: &mut H, renderer: &mut R
    ) {
        self.update_tick_config(renderer);
        handler.init(self.handle(renderer));
    }

    pub fn destroy<H: Handler<E, S, L, R>>(
        &mut self, handler: &mut H, renderer: &mut R
    ) {

        handler.destroy(self.handle(renderer));

        // Send any pending outgoing events
        self.send_events();
        self.network.flush().unwrap();
        self.network.close().unwrap();

    }


    // Tick Handling ----------------------------------------------------------
    pub fn tick<H: Handler<E, S, L, R>>(
        &mut self, handler: &mut H, renderer: &mut R

    ) -> bool {

        let mut ticked = false;

        while let Ok(event) = self.network.receive() {
            match event {

                ClientEvent::Connection => {
                    self.remote_states.clear();
                    self.manager.reset();
                    handler.connect(self.handle(renderer));
                },

                ClientEvent::Message(data) =>  {
                    match network::Message::from_u8(data[0]) {

                        network::Message::ServerConfig => {
                            let data = self.manager.receive_config(&data[1..]);
                            self.update_tick_config(renderer);
                            handler.config(self.handle(renderer), data);
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
                            handler.event(self.handle(renderer), owner, event);
                        }
                    }

                    handler.tick_before(self.handle(renderer));
                    self.tick_entities(handler, renderer);
                    self.send_events();
                    handler.tick_after(self.handle(renderer));
                    ticked = true;

                },

                ClientEvent::Close | ClientEvent::ConnectionLost => {
                    self.remote_states.clear();
                    self.manager.reset();
                    handler.disconnect(self.handle(renderer), true);
                },

                ClientEvent::ConnectionFailed => {
                    handler.disconnect(self.handle(renderer), false);
                },

                _ => {}

            }
        }

        self.network.flush().unwrap();

        ticked

    }

    pub fn draw<H: Handler<E, S, L, R>>(
        &mut self, handler: &mut H, renderer: &mut R
    ) {
        handler.draw(self.handle(renderer));
    }


    // Internal ---------------------------------------------------------------
    fn handle<'a>(&'a mut self, renderer: &'a mut R) -> Handle<E, S, L, R> {
        Handle {
            renderer: renderer,
            level: &mut self.level,
            events: &mut self.events,
            entities: &mut self.manager,
            network: &mut self.network
        }
    }

    fn update_tick_config(&mut self, renderer: &mut R) {
        let tick_rate = self.manager.config().tick_rate as u32;
        let config = self.network.config();
        self.network.set_config(Config {
            send_rate: tick_rate,
            .. config
        });
        renderer.set_tick_rate(tick_rate);
        renderer.set_interpolation_ticks(
            self.manager.config().interpolation_ticks as usize
        );
    }

    fn tick_entities<H: Handler<E, S, L, R>>(
        &mut self, handler: &mut H, renderer: &mut R
    ) {

        let local_inputs = self.manager.tick_client(
            renderer, handler, &self.level
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

    fn send_message(&mut self, kind: MessageKind, typ: network::Message, data: &Vec<u8>) {
        let mut msg = [typ as u8].to_vec();
        msg.extend(data);
        self.network.send(kind, msg).unwrap();
    }

}


// Client Handle for Access from Handler --------------------------------------
pub struct Handle<
    'a,
    E: Event + 'a,
    S: EntityState + 'a,
    L: BaseLevel<S> + 'a,
    R: Renderer + 'a
> {
    pub renderer: &'a mut R,
    pub level: &'a mut Level<S, L>,
    pub events: &'a mut EventHandler<E>,
    pub entities: &'a mut EntityManager<S, L, R>,
    pub network: &'a mut ClientStream
}


// Client Handler -------------------------------------------------------------
pub trait Handler<E: Event, S: EntityState, L: BaseLevel<S>, R: Renderer> {

    fn init(&mut self, Handle<E, S, L, R>);
    fn connect(&mut self, Handle<E, S, L, R>);
    fn disconnect(&mut self, Handle<E, S, L, R>, bool);

    fn config(&mut self, Handle<E, S, L, R>, &[u8]);

    fn event(&mut self, Handle<E, S, L, R>, ConnectionID, E);

    fn tick_before(&mut self, Handle<E, S, L, R>);

    fn tick_entity_before(
        &mut self, &mut R, &Level<S, L>, &mut Entity<S, L, R>, u8, f32
    );

    fn tick_entity_after(
        &mut self, &mut R, &Level<S, L>, &mut Entity<S, L, R>, u8, f32
    );

    fn tick_after(&mut self, Handle<E, S, L, R>);

    fn draw(&mut self, Handle<E, S, L, R>);

    fn destroy(&mut self, Handle<E, S, L, R>);

}

