#[macro_use]
extern crate clap;

#[macro_use]
extern crate allegro;
extern crate allegro_primitives;
extern crate allegro_sys;

use allegro::*;
use allegro_primitives::*;

mod game;

allegro_main! {

    let args = clap::App::new("client")
        .version(&crate_version!())
        .author("Ivo Wetzel <ivo.wetzel@googlemail.com>")
        .about("Server")
        .arg(clap::Arg::with_name("address:port")
            .help("Remote server address to connect to.")
            .index(1)
        ).get_matches();

    let mut core = Core::init().unwrap();
    core.set_new_display_option(DisplayOption::SampleBuffers, 1, DisplayOptionImportance::Suggest);
    core.set_new_display_option(DisplayOption::Samples, 8, DisplayOptionImportance::Suggest);

    let disp = Display::new(&core, 600, 600).unwrap();
	disp.set_window_title("Rustgame: Shooter");

	core.install_keyboard().unwrap();
	core.install_mouse().unwrap();

    let timer = Timer::new(&core, 1.0 / 60.0).unwrap();

    let q = EventQueue::new(&core).unwrap();
	q.register_event_source(disp.get_event_source());
	q.register_event_source(core.get_keyboard_event_source());
	q.register_event_source(core.get_mouse_event_source());
	q.register_event_source(timer.get_event_source());
    timer.start();

    let prim = PrimitivesAddon::init(&core).unwrap();

    let tick_step = 2;
    let tick_dt = 1.0 / (60.0 / tick_step as f32);

    let mut key_state: [bool; 255] = [false; 255];
    let mut delay = 0;
    let mut tick: i32 = 0;
    let mut logic_time: f64 = 0.0;

    let mut redraw = true;
    let mut last_render_time: f64 = 0.0;
    let mut render_time: f64 = 0.0;

    let mut game = game::Game::new(&core);

    'exit: loop {

        if redraw && q.is_empty() {

            let dt = render_time - last_render_time;
            let u = 1.0 / (tick_dt as f64) * (render_time - logic_time);

            game.draw(&core, &prim, dt as f32, u as f32);
            disp.flip();
            redraw = false;

        }

        match q.wait_for_event() {

			DisplayClose{source: src, ..} => {
				assert!(disp.get_event_source().get_event_source() == src);
				break 'exit;
			},

			KeyDown{keycode: k, ..} if (k as u32) < 255 => {
                key_state[k as usize] = true;
			},

			KeyUp{keycode: k, ..} if (k as u32) < 255 => {
                key_state[k as usize] = false;
			},

            TimerTick{timestamp: frame_time, ..} => {

                last_render_time = render_time;
                render_time = frame_time;

                if delay == tick_step {
                    delay = 0;
                    logic_time = frame_time;
                    game.tick(&key_state, tick as u8, tick_dt);
                    tick = (tick + 1) % 256;
                }

                delay += 1;
                redraw = true

            },

            _ => () // println!("Some other event...")

        }

    }

}

