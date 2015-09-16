#[macro_use]
extern crate allegro;
extern crate allegro_primitives;
extern crate rand;
extern crate allegro_sys;

use allegro::*;
use allegro_primitives::*;
use particle::ParticleSystem;
use rand::Rng;

mod color;
mod particle;
mod ship;

allegro_main! {

    let mut core = Core::init().unwrap();

    core.set_new_display_option(DisplayOption::SampleBuffers, 1, DisplayOptionImportance::Suggest);
    core.set_new_display_option(DisplayOption::Samples, 8, DisplayOptionImportance::Suggest);

    let disp = Display::new(&core, 600, 600).unwrap();
	disp.set_window_title("Rustgame: Shooter");

	core.install_keyboard().unwrap();
	core.install_mouse().unwrap();

    let prim = PrimitivesAddon::init(&core).unwrap();

    let timer = Timer::new(&core, 1.0 / 60.0).unwrap();

    let q = EventQueue::new(&core).unwrap();
	q.register_event_source(disp.get_event_source());
	q.register_event_source(core.get_keyboard_event_source());
	q.register_event_source(core.get_mouse_event_source());
	q.register_event_source(timer.get_event_source());

    timer.start();

    let back_color = core.map_rgb_f(0.0, 0.0, 0.0);

    let mut players = vec![
        ship::PlayerShip::new(60.0, 60.0, color::Color::red())
        //ship::PlayerShip::new(140.0, 60.0, color::Color::orange()),
        //ship::PlayerShip::new(220.0, 60.0, color::Color::yellow()),
        //ship::PlayerShip::new(300.0, 60.0, color::Color::green()),
        //ship::PlayerShip::new(60.0, 140.0, color::Color::teal()),
        //ship::PlayerShip::new(140.0, 140.0, color::Color::blue()),
        //ship::PlayerShip::new(220.0, 140.0, color::Color::purple()),
        //ship::PlayerShip::new(300.0, 140.0, color::Color::pink())
    ];

    let mut active_ship = 0;
    let mut rng = rand::XorShiftRng::new_unseeded();
    let mut key_state: [bool; 255] = [false; 255];

    let mut redraw = true;
    let mut tick: i32 = 0;
    let mut remote_states: Vec<(u8, ship::ShipState)> = Vec::new();

    'exit: loop {

        if redraw && q.is_empty() {

            let dt = 1.0 / 60.0;
            core.clear_to_color(back_color);

            for p in players.iter_mut() {
                p.draw(&core, &prim, dt);
            }

            //particle_system.draw(&core, &prim, dt);

            disp.flip();
            redraw = false;

        }

        match q.wait_for_event() {

			DisplayClose{source: src, ..} => {
				assert!(disp.get_event_source().get_event_source() == src);
				println!("Display close event...");
				break 'exit;
			},

			KeyDown{keycode: k, ..} if (k as u32) < 255 => {
                key_state[k as usize] = true;
                //println!("{}", k as u32);
			},

			KeyUp{keycode: k, ..} if (k as u32) < 255 => {
                key_state[k as usize] = false;
			},

            TimerTick{..} => {

                let dt = 1.0 / 60.0;

                for (i, p) in players.iter_mut().enumerate() {

                    if i == active_ship {
                        p.input(ship::Input {
                            tick: tick as u8,
                            left: key_state[1],
                            right: key_state[4],
                            thrust: key_state[23],
                            fire: false
                        });

                        // Emulate remote server state stuff with a 20 frames
                        // delay
                        if remote_states.len() > 20 {
                            let first = remote_states.remove(0);
                            p.apply_remote_state(first.0, first.1)
                        }

                        p.step(&mut rng, dt);

                        remote_states.push((tick as u8, p.get_state()));

                    } else {
                        p.step(&mut rng, dt);
                    }

                }

                tick = (tick + 1) % 256;
                redraw = true

            },

	        MouseButtonDown{button: b, ..} => {
				println!("Mouse button {} pressed", b);
			},

            _ => () // println!("Some other event...")

        }

    }

}

