// External Dependencies ------------------------------------------------------
use std::net::SocketAddr;
use cobalt::MessageKind;


// Internal Dependencies ------------------------------------------------------
use entity;
use event;
use level;
use network;
use renderer::Renderer;
use runnable::Runnable;


// Client Abstraction ---------------------------------------------------------
pub struct Client<E, L> where E: event::Event, L: level::Level {
    network: network::Stream,
    manager: entity::Manager,
    events: event::Handler<E>,
    remote_states: Vec<(u8, entity::State)>,
    level: L
}

impl<E, L> Client<E, L> where E: event::Event, L: level::Level {

    pub fn new(server_addr: SocketAddr, level: L, registry: Box<entity::Registry>) -> Client<E, L> {
        Client {
            // TODO make initial address optional?
            network: network::Stream::new(server_addr),
            manager: entity::Manager::new(
                30, 1000, 75,
                false,
                registry
            ),
            events: event::Handler::new(),
            remote_states: Vec::new(),
            level: level
        }
    }

    pub fn init(&mut self, runnable: &mut Runnable<E, L>, renderer: &mut Renderer) {
        self.network.set_tick_rate(self.manager.config().tick_rate as u32);
        self.manager.init(renderer);
        runnable.init(self.proxy(renderer));
    }

    pub fn destroy(&mut self, runnable: &mut Runnable<E, L>, renderer: &mut Renderer) {
        self.network.destroy();
        runnable.destroy(self.proxy(renderer));
    }

    fn proxy<'a>(&'a mut self, renderer: &'a mut Renderer) -> ClientProxy<L> {
        ClientProxy {
            renderer: renderer,
            level: &self.level,
            entities: &mut self.manager,
            network: &self.network
        }
    }


    // Tick Handling ----------------------------------------------------------
    pub fn tick(
        &mut self,
        runnable: &mut Runnable<E, L>,
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
                    runnable.connect(self.proxy(renderer));
                },

                // TODO clean up / validate message
                network::StreamEvent::Message(_, data) =>  {
                    match network::Message::from_u8(data[0]) {
                        network::Message::ServerConfig => {
                            let level_data = self.manager.receive_config(renderer, &data[1..]);
                            self.network.set_tick_rate(self.manager.config().tick_rate as u32);
                            self.level = runnable.level(self.proxy(renderer), level_data);
                            runnable.config(self.proxy(renderer));
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
                        for (_, event) in events {
                            runnable.event(self.proxy(renderer), event);
                        }
                    }

                    let tick = self.manager.tick();
                    runnable.tick_before(self.proxy(renderer), tick, dt);
                    self.tick_entities(dt, runnable, renderer);
                    runnable.tick_after(self.proxy(renderer), tick, dt);
                    ticked = true;

                },

                network::StreamEvent::Close | network::StreamEvent::ConnectionLost(_) => {
                    self.remote_states.clear();
                    self.manager.reset();
                    runnable.disconnect(self.proxy(renderer));
                },

                _ => {}

            }
        }

        self.network.send();

        ticked

    }

    pub fn draw(&mut self, runnable: &mut Runnable<E, L>, renderer: &mut Renderer) {
        runnable.draw(self.proxy(renderer));
    }

    pub fn tick_entities(
        &mut self,
        dt: f32,
        runnable: &mut Runnable<E, L>,
        renderer: &mut Renderer
    ) {

        let mut local_inputs: Option<Vec<u8>> = None;
        let remote_states = &mut self.remote_states;

        // Tick entities
        self.manager.tick_client_entities(
            renderer, runnable,
            &self.level, dt,
            |state, entity, tick| {
                match state {

                    // TODO this will no longer be needed once we have a local
                    // network proxy
                    entity::ControlState::Remote => {
                        local_inputs = entity.serialized_inputs();
                    },

                    // TODO solve this via a local network proxy which has a delay
                    entity::ControlState::Local => {

                        // Emulate remote server state stuff with a 20 frames delay
                        remote_states.push((tick, entity.state().clone()));

                        if remote_states.len() > 20 {
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
        if let Some(ref events) = self.events.serialize_events() {
            let mut data = [network::Message::ClientEvents as u8].to_vec();
            data.extend(events.clone());
            self.network.send_message(MessageKind::Reliable, data);
        }

    }

}


// Client Proxy for Access from Runnable --------------------------------------
pub struct ClientProxy<'a, 'b, L: 'a, > {
    pub renderer: &'b mut Renderer,
    pub level: &'a L,
    pub entities: &'a mut entity::Manager,
    pub network: &'a network::Stream
}

