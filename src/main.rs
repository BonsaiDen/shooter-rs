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
	core.install_joystick().unwrap();

    let prim = PrimitivesAddon::init(&core).unwrap();

    let timer = Timer::new(&core, 1.0 / 60.0).unwrap();

    let q = EventQueue::new(&core).unwrap();
	q.register_event_source(disp.get_event_source());
	q.register_event_source(core.get_keyboard_event_source());
	q.register_event_source(core.get_mouse_event_source());
	q.register_event_source(core.get_joystick_event_source());
	q.register_event_source(timer.get_event_source());

    timer.start();

    let back_color = core.map_rgb_f(0.0, 0.0, 0.0);

    let mut players = vec![
        ship::DrawableShip::new(60.0, 60.0, color::Color::red()),
        ship::DrawableShip::new(140.0, 60.0, color::Color::orange()),
        ship::DrawableShip::new(220.0, 60.0, color::Color::yellow()),
        ship::DrawableShip::new(300.0, 60.0, color::Color::green()),
        ship::DrawableShip::new(60.0, 140.0, color::Color::teal()),
        ship::DrawableShip::new(140.0, 140.0, color::Color::blue()),
        ship::DrawableShip::new(220.0, 140.0, color::Color::purple()),
        ship::DrawableShip::new(300.0, 140.0, color::Color::pink())
    ];

    let mut active_ship = 0;
    let mut rng = rand::XorShiftRng::new_unseeded();
    let mut key_state: [bool; 255] = [false; 255];
    //let mut particle_system = ParticleSystem::new(10000);

    let mut redraw = true;
    let mut jx = 0.0;
    let mut jy = 0.0;
    let mut jr = 0.0;

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

            JoystickAxes{stick: s, axis: a, pos: p, ..} => {
                if s == 0 {

                    if a == 0 {
                        jx = p;
                        //key_state[1] = p < -0.1;
                        //key_state[4] = p > 0.1;

                    } else if a == 1 {
                        jy = p;
                    }

                    if jx.abs() + jy.abs() > 0.9 {
                        //jr = jy.atan2(jx);

                    } else {
                        jr = 255.0;
                    }

                }
            },

            JoystickButtonDown{button: b, ..} => {
                if b == 0 {
                    key_state[23] = true;
                } else if (b == 1) {
                    active_ship = (active_ship + 1) % 8;
                }
            },

            JoystickButtonUp{button: b, ..} => {
                if b == 0 {
                    key_state[23] = false;
                }
            },

            // TODO accumulate actual delta time
            TimerTick{..} => {

                let dt = 1.0 / 60.0;

                for (i, p) in players.iter_mut().enumerate() {

                    let mut steer = 0.0;
                    let mut thrust = false;

                    if i == active_ship {

                        if key_state[1] {
                            steer -= 1.0;
                        }

                        if key_state[4] {
                            steer += 1.0;
                        }

                        thrust = key_state[23];

                        p.step(&mut rng, dt, steer, thrust, jr);

                    } else {
                        p.step(&mut rng, dt, steer, thrust, 0.0);
                    }

                }

                /*
                for _ in 0..10 {
                    if let Some(p) = particle_system.get() {
                        p.color = color::Color::yellow();
                        p.x = 200.0;
                        p.y = 200.0;
                        p.s = 2.5;
                        p.sms = -0.25;
                        p.v = (86.0 + rng.gen::<u8>() as f32 / 9.0) * 0.5;
                        p.vms = 0.0;
                        p.r = (255.0 / (rng.gen::<u8>() as f32)) * std::f32::consts::PI * 2.0;
                        p.rms = 0.0;
                        p.fadeout = 0.25;
                        p.lifetime = 14.0;//2.0 + (rng.gen::<u8>() as f32) / 128.0;
                        p.remaining = p.lifetime;
                    }
                }
                */

                redraw = true

            },

	        MouseButtonDown{button: b, ..} => {
				println!("Mouse button {} pressed", b);
			},

            _ => () // println!("Some other event...")

        }

    }

}

