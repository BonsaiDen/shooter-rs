//use std::net::SocketAddr;
use rand::{SeedableRng, XorShiftRng};
use allegro::{Core, Color};
use allegro_primitives::PrimitivesAddon;
use allegro_font::{Font, FontAlign, FontDrawing};

use shared;
use net;

pub struct Game {
    rng: XorShiftRng,
    back_color: Color,
    text_color: Color,
    ships: Vec<shared::ship::PlayerShip>,
    remote_states: Vec<(u8, shared::ship::ShipState)>,
    arena: shared::arena::Arena
}

impl Game {

    pub fn new(core: &Core) -> Game {
        Game {
            rng: XorShiftRng::new_unseeded(),
            back_color: core.map_rgb(0, 0, 0),
            text_color: core.map_rgb(255, 255, 255),
            ships: Vec::new(),
            remote_states: Vec::new(),
            arena: shared::arena::Arena::new(768, 768, 16)
        }
    }

    pub fn init(&mut self, core: &Core) {
        self.ships.push(
            shared::ship::PlayerShip::new(
                60.0, 60.0, true, shared::color::Color::red()
            )
        );
        // TODO implement network and events
        // self.network.send(NetworkEvent::JoinRequest());
    }

    pub fn draw(
        &mut self, core: &Core, prim: &PrimitivesAddon, font: &Font,
        network: &mut net::Network,
        dt: f32, u: f32
    ) {

        core.clear_to_color(self.back_color);

        for s in self.ships.iter_mut() {
            s.draw(&core, &prim, &mut self.rng, dt, u);
        }

        // UI
        let network_state = match network.connected() {
            true => format!("Connected to server at {}", network.server_addr()),
            false => format!("Connecting to server at {}...", network.server_addr())
        };
        core.draw_text(font, self.text_color, 0.0, 0.0, FontAlign::Left, &network_state[..]);

    }

    pub fn tick(
        &mut self, network: &mut net::Network, key_state: &[bool; 255],
        initial_tick: bool, tick: u8, dt: f32,
    ) {

        // TODO bullets are handled by pre-creating a local object and then
        // syncing it with the remote one, we submit a local ID and the server
        // return this ID along with the remote object ID when updating

        // TODO server side
        // - send full state or diff from last confirmed local tick?

        // TODO implement network and events
        for s in self.ships.iter_mut() {

            if s.is_local() {

                let input = shared::ship::Input {
                    tick: tick as u8,
                    left: key_state[1] || key_state[82],
                    right: key_state[4] || key_state[83],
                    thrust: key_state[23] || key_state[84],
                    fire: false
                };

                s.input(input);

                // Emulate remote server state stuff with a 20 frames delay
                if self.remote_states.len() > 20 {
                    // TODO apply the states received from the server
                    let first = self.remote_states.remove(0);
                    s.remote_step(dt, initial_tick, first.0, first.1);

                } else {
                    s.step(dt, initial_tick);
                }

                // TODO send input to server
                self.remote_states.push((tick, s.get_state()));

                // TODO implement network event
                // self.network.send(NetworkEvent::Input(input))

                // TODO server side collision is checked on each server tick
                // positions are warped to the last known local tick of the player
                // BUT there is a maximum tick difference to prevent cheating

            } else {
                s.step(dt, initial_tick);
            }

        }

        self.rng.reseed([
            ((tick as u32 + 7) * 941) as u32,
            ((tick as u32 + 659) * 461) as u32,
            ((tick as u32 + 13) * 227) as u32,
            ((tick as u32 + 97) * 37) as u32
        ]);

    }

}

