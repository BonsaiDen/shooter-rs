// External Dependencies ------------------------------------------------------
use lithium::entity;
use lithium::renderer::Renderer;
use lithium::runnable::Runnable;


// Internal Dependencies ------------------------------------------------------
use net;
use game::{Game, State};
use renderer::AllegroRenderer;
use shared::color::{Color, ColorName};


// Runnable Implementation ----------------------------------------------------
impl Runnable for Game {

    fn init(&mut self, renderer: &mut Renderer) {

        // TODO clean up!
        self.network.set_tick_rate(self.manager.config().tick_rate as u32);
        self.manager.init(renderer);
        renderer.set_fps(60);

        let ar = AllegroRenderer::downcast_mut(renderer);
        ar.set_title("Rustgame: Shooter");
        ar.resize(self.level.width() as i32, self.level.height() as i32);

        // Local Test Play
        if self.network.connected() == false {

            let (x, y) = self.level.center();
            let flags = 0b0000_0001 | Color::from_name(ColorName::Red).to_flags();
            let state = entity::State {
                x: x as f32,
                y: y as f32,
                flags: flags,
                .. entity::State::default()
            };

            self.manager.create_entity(0, Some(state), None);

        }

    }

    fn tick(&mut self, renderer: &mut Renderer) -> bool {

        let mut ticked = false;
        let tick_rate = self.network.get_tick_rate();
        let dt = 1.0 / tick_rate as f32;

        self.network.receive();

        while let Ok(event) = self.network.message(renderer.time()) {
            match event {

                net::EventType::Connection(_) => {
                    self.connect();
                },

                net::EventType::Message(_, data) =>  {
                    // TODO validate message length
                    if data.len() > 0 {
                        match self.state {
                            State::Pending => {

                                // Game Configuration
                                // TODO use enum for type
                                if data[0] == 0 {
                                    self.config(renderer, &data[1..]);
                                }

                            },
                            State::Connected => {
                                // Game State
                                // TODO use enum for type
                                if data[0] == 1 {
                                    self.manager.receive_state(&data[1..]);
                                }
                            },
                            _ => {}
                        }
                    }
                },

                net::EventType::Tick(_, _, _) => {
                    ticked = true;
                    self.tick_entities(renderer, dt);
                },

                net::EventType::Close => {
                    println!("Connection closed");
                },

                net::EventType::ConnectionLost(_) => {
                    self.disconnect(renderer);
                },

                _ => {}

            }
        }

        self.network.send();

        ticked

    }

    fn draw(&mut self, renderer: &mut Renderer) {

        AllegroRenderer::downcast_mut(renderer).clear(&self.back_color);

        self.manager.draw_entities(renderer, &self.level);

        if let Ok(addr) = self.network.server_addr() {
            let network_state = match self.network.connected() {
                true => format!(
                    "{} (Ping: {}ms, Lost: {:.2}%, Bytes: {}/{})",
                    addr,
                    self.network.rtt() / 2,
                    self.network.packet_loss(),
                    self.network.bytes_sent(),
                    self.network.bytes_received()
                ),
                false => format!("Connecting to {}...", addr)
            };

            AllegroRenderer::downcast_mut(renderer).text(
                &self.text_color, 0.0, 0.0, &network_state[..]
            );

        }

    }

    fn destroy(&mut self) {
        self.network.destroy();
    }

}

