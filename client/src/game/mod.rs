// External Dependencies ------------------------------------------------------
use std::net::SocketAddr;
use lithium::entity;
use lithium::event;
use lithium::renderer::Renderer;
use lithium::runnable::Runnable;


// Internal Dependencies ------------------------------------------------------
use net;
use entities;
use renderer::AllegroRenderer;
use shared::NetworkMessage;
use shared::event::Event;
use shared::level::Level;
mod runnable;

enum State {
    Disconnected,
    Pending,
    Connected
}

// Game -----------------------------------------------------------------------
pub struct Game {
    network: net::Network,
    manager: entity::Manager,
    events: event::Handler<Event>,
    remote_states: Vec<(u8, entity::State)>,
    level: Level,
    state: State
}

impl Game {

    pub fn new(server_addr: SocketAddr) -> Game {
        Game {
            manager: entity::Manager::new(
                30, 1000, 75,
                false,
                Box::new(entities::Registry)
            ),
            events: event::Handler::new(),
            remote_states: Vec::new(),
            network: net::Network::new(server_addr),
            level: Game::default_level(),
            state: State::Disconnected
        }
    }

    pub fn default_level() -> Level {
        Level::new(384, 384, 16)
    }


    // Internal ---------------------------------------------------------------
    fn connect(&mut self) {
        self.state = State::Pending;
        self.reset();
    }

    fn disconnect(&mut self, renderer: &mut Renderer) {
        self.level = Game::default_level();
        self.state = State::Disconnected;
        self.reset();
        self.init(renderer);
    }

    fn reset(&mut self) {
        self.remote_states.clear();
        self.manager.reset();
    }

    fn config(&mut self, renderer: &mut Renderer, data: &[u8]) {
        let level_data = self.manager.receive_config(renderer, data);
        self.network.set_tick_rate(self.manager.config().tick_rate as u32);
        self.level = Level::from_serialized(level_data);
        self.state = State::Connected;
        self.init(renderer);
    }

    fn event(&mut self, event: Event) {
        println!("Event: {:?}", event);
    }

    fn tick_entities(&mut self, renderer: &mut Renderer, dt: f32) {

        // TODO clean up
        let ar = AllegroRenderer::downcast_mut(renderer);
        let tick = self.manager.tick();
        ar.reseed_rng([
            ((tick as u32 + 7) * 941) as u32,
            ((tick as u32 + 659) * 461) as u32,
            ((tick as u32 + 13) * 227) as u32,
            ((tick as u32 + 97) * 37) as u32
        ]);

        let state = &self.state;
        let remote_states = &mut self.remote_states;
        let mut local_inputs = None;

        self.manager.tick_entities(&self.level, dt, |entity, _, tick, _| {

            if entity.local() {

                let mut buttons = 0;
                if ar.key_down(1) || ar.key_down(82) {
                    buttons |= 0x01;
                }

                if ar.key_down(4) || ar.key_down(83) {
                    buttons |= 0x02;
                }

                if ar.key_down(23) || ar.key_down(84) {
                    buttons |= 0x04;
                }

                let input = entity::Input {
                    tick: tick,
                    fields: buttons
                };

                entity.local_input(input);

            }

        }, |entity, _, tick, _| {

            // TODO clean up
            if entity.local() {
                match *state {
                    State::Disconnected => {

                        // Emulate remote server state stuff with a 20 frames delay
                        remote_states.push((tick, entity.state().clone()));

                        if remote_states.len() > 20 {
                            let first = remote_states.remove(0);
                            entity.set_confirmed_state(first.0, first.1);
                        }

                    },

                    State::Connected => {
                        local_inputs = entity.serialized_inputs();
                    },

                    _ => {}

                }
            }

        });

        // Send all unconfirmed inputs to server
        if let Some(inputs) = local_inputs {
            // TODO create a fake local network proxy!
            let mut data = [NetworkMessage::ClientInput as u8].to_vec();
            data.extend(inputs);
            self.network.send_message(net::MessageKind::Instant, data);
        }

        // Send events
        if let Some(ref events) = self.events.serialize_events() {
            let mut data = [NetworkMessage::ClientEvents as u8].to_vec();
            data.extend(events.clone());
            self.network.send_message(net::MessageKind::Reliable, data);
        }

    }

}

