extern crate rand;
extern crate shared;

use self::rand::SeedableRng;

use allegro::{Core, Color};
use allegro_primitives::PrimitivesAddon;

pub struct Game {
    rng: rand::XorShiftRng,
    back_color: Color,
    ships: Vec<shared::ship::PlayerShip>,
    remote_states: Vec<(u8, shared::ship::ShipState)>
}

impl Game {

    pub fn new(core: &Core) -> Game {

        let mut game = Game {
            rng: rand::XorShiftRng::new_unseeded(),
            back_color: core.map_rgb_f(0.0, 0.0, 0.0),
            ships: Vec::new(),
            remote_states: Vec::new()
        };

        game.ships.push(
            shared::ship::PlayerShip::new(
                60.0, 60.0, true, shared::color::Color::red()
            )
        );

        game

    }

    pub fn draw(&mut self, core: &Core, prim: &PrimitivesAddon, dt: f32, u: f32) {
        core.clear_to_color(self.back_color);
        for s in self.ships.iter_mut() {
            s.draw(&core, &prim, &mut self.rng, dt, u);
        }
    }

    pub fn tick(&mut self, key_state: &[bool; 255], tick: u8, dt: f32) {

        for s in self.ships.iter_mut() {

            if s.is_local() {

                s.input(shared::ship::Input {
                    tick: tick as u8,
                    left: key_state[1],
                    right: key_state[4],
                    thrust: key_state[23],
                    fire: false
                });

                // Emulate remote server state stuff with a 20 frames delay
                if self.remote_states.len() > 20 {
                    // TODO apply the states received from the server
                    let first = self.remote_states.remove(0);
                    s.remote_step(dt, first.0, first.1);

                } else {
                    s.step(dt);
                }

                // TODO send input to server
                self.remote_states.push((tick, s.get_state()));

            } else {
                s.step(dt);
            }

        }

        self.rng.reseed([
            ((tick as u32 + 7) * 941) as u32,
            ((tick as u32 + 13) * 227) as u32,
            ((tick as u32 + 97) * 37) as u32,
            ((tick as u32 + 659) * 461) as u32
        ]);

    }

}

