// External Dependencies ------------------------------------------------------
use std::net::SocketAddr;
use cobalt::{MessageKind, ConnectionID};


// Internal Dependencies ------------------------------------------------------
use network;
use entity::{
    Entity,
    State,
    Manager as EntityManager,
    Registry as EntityRegistry,
    ControlState as EntityControlState
};
use level::Level;
use event::{Event, Handler as EventHandler};
use renderer::Renderer;


// Client Abstraction ---------------------------------------------------------
pub struct Client<E: Event, S: State> {
    network: network::Stream,
    manager: EntityManager<S>,
    events: EventHandler<E>,
    remote_states: Vec<(u8, S)>,
    level: Level<S>
}

impl<E: Event, S: State> Client<E, S> {

    // Statics ----------------------------------------------------------------
    pub fn new(
        server_addr: SocketAddr,
        tick_rate: u8,
        level: Level<S>,
        registry: Box<EntityRegistry<S>>

    ) -> Client<E, S> {

        let mut network = network::Stream::new(server_addr);
        network.set_tick_rate(tick_rate as u32);

        Client {
            network: network,
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
    pub fn init(&mut self, handler: &mut Handler<E, S>, renderer: &mut Renderer) {
        self.network.set_tick_rate(self.manager.config().tick_rate as u32);
        renderer.set_tick_rate(self.manager.config().tick_rate as u32);
        renderer.set_interpolation_ticks(
            self.manager.config().interpolation_ticks as usize
        );
        handler.init(self.handle(renderer));
    }

    pub fn destroy(&mut self, handler: &mut Handler<E, S>, renderer: &mut Renderer) {
        self.network.destroy();
        handler.destroy(self.handle(renderer));
    }


    // Tick Handling ----------------------------------------------------------
    pub fn tick(
        &mut self,
        handler: &mut Handler<E, S>,
        renderer: &mut Renderer

    ) -> bool {

        let mut ticked = false;
        let tick_rate = self.manager.config().tick_rate;
        let dt = 1.0 / tick_rate as f32;

        self.network.receive();

        while let Ok(event) = self.network.message(renderer.time()) {
            match event {

                network::StreamEvent::Connection(_) => {
                    self.remote_states.clear();
                    self.manager.reset();
                    handler.connect(self.handle(renderer));
                },

                // TODO clean up / validate message
                network::StreamEvent::Message(_, data) =>  {
                    match network::Message::from_u8(data[0]) {
                        network::Message::ServerConfig => {
                            let level_data = self.manager.receive_config(&data[1..]);
                            self.level = handler.level(self.handle(renderer), level_data);
                            self.network.set_tick_rate(self.manager.config().tick_rate as u32);
                            renderer.set_tick_rate(self.manager.config().tick_rate as u32);
                            renderer.set_interpolation_ticks(self.manager.config().interpolation_ticks as usize);
                            handler.config(self.handle(renderer));
                        },
                        network::Message::ServerState => {
                            self.manager.receive_state(&data[1..]);
                        },
                        network::Message::ServerEvents => {
                            self.events.receive_events(self.network.id(), &data[1..]);
                        },
                        _=> println!("Unknown Server Message {:?}", data)
                    }
                },

                network::StreamEvent::Tick(_, _, _) => {

                    if let Some(events) = self.events.received() {
                        for (owner, event) in events {
                            handler.event(self.handle(renderer), owner, event);
                        }
                    }

                    let tick = self.manager.tick();
                    handler.tick_before(self.handle(renderer), tick, dt);
                    self.tick_entities(dt, handler, renderer);
                    handler.tick_after(self.handle(renderer), tick, dt);
                    ticked = true;

                },

                network::StreamEvent::Close | network::StreamEvent::ConnectionLost(_) => {
                    self.remote_states.clear();
                    self.manager.reset();
                    handler.disconnect(self.handle(renderer));
                },

                _ => {}

            }
        }

        self.network.send();

        ticked

    }

    pub fn draw(
        &mut self, handler: &mut Handler<E, S>, renderer: &mut Renderer
    ) {
        handler.draw(self.handle(renderer));
    }


    // Internal ---------------------------------------------------------------
    fn handle<'a>(&'a mut self, renderer: &'a mut Renderer) -> Handle<E, S> {
        Handle {
            renderer: renderer,
            level: &mut self.level,
            events: &mut self.events,
            entities: &mut self.manager,
            network: &self.network
        }
    }

    fn tick_entities(
        &mut self,
        dt: f32,
        handler: &mut Handler<E, S>,
        renderer: &mut Renderer
    ) {

        let mut local_inputs: Option<Vec<u8>> = None;
        let remote_states = &mut self.remote_states;

        // Tick entities
        self.manager.tick_client(
            renderer, handler,
            &self.level, dt,
            |state, entity, tick| {
                match state {

                    // TODO this will no longer be needed once we have a local
                    // network proxy
                    EntityControlState::Remote => {
                        local_inputs = entity.serialized_inputs();
                    },

                    // TODO solve this via a local network proxy which has a delay
                    EntityControlState::Local => {

                        // Emulate remote server state stuff with a 20 frames delay
                        remote_states.push((tick, entity.state().clone()));

                        // TODO scale delay with tick rate or configure it
                        // TODO or increase state buffer size?
                        if remote_states.len() > 10 {
                            let first = remote_states.remove(0);
                            entity.set_confirmed_state(first.0, first.1);
                        }

                    },
                    _ => unreachable!()
                }
            }
        );

        // Send all unconfirmed inputs to server
        if let Some(inputs) = local_inputs {
            let mut data = [network::Message::ClientInput as u8].to_vec();
            data.extend(inputs);
            self.network.send_message(MessageKind::Instant, data);
        }

        // Send events
        if let Some(ref events) = self.events.serialize_events(None) {
            let mut data = [network::Message::ClientEvents as u8].to_vec();
            data.extend(events.clone());
            self.network.send_message(MessageKind::Reliable, data);
        }

        self.events.flush();

    }

}


// Client Handle for Access from Handler ------------------------------------
pub struct Handle<
    'a, 'b,
    E: Event + 'a,
    S: State + 'a
> {
    pub renderer: &'b mut Renderer,
    pub level: &'a mut Level<S>,
    pub events: &'a mut EventHandler<E>,
    pub entities: &'a mut EntityManager<S>,
    pub network: &'a network::Stream
}


// Client Handler -------------------------------------------------------------
pub trait Handler<E: Event, S: State> {

    fn init(&mut self, Handle<E, S>);
    fn connect(&mut self, Handle<E, S>);
    fn disconnect(&mut self, Handle<E, S>);

    fn level(&mut self, Handle<E, S>, &[u8]) -> Level<S>;
    fn config(&mut self, Handle<E, S>);

    fn event(&mut self, Handle<E, S>, ConnectionID, E);
    fn tick_before(&mut self, Handle<E, S>, u8, f32);

    fn tick_entity_before(
        &mut self, &mut Renderer, &Level<S>, &mut Entity<S>, u8, f32
    );

    fn tick_entity_after(
        &mut self, &mut Renderer, &Level<S>, &mut Entity<S>, u8, f32

    ) -> EntityControlState;

    fn tick_after(&mut self, Handle<E, S>, u8, f32);

    fn draw(&mut self, Handle<E, S>);

    fn destroy(&mut self, Handle<E, S>);

}

