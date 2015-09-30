#[macro_use]
extern crate clap;
extern crate rand;
extern crate cobalt;
extern crate clock_ticks;
extern crate shared;

#[macro_use]
extern crate allegro;
extern crate allegro_sys;
extern crate allegro_font;
extern crate allegro_primitives;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use allegro::*;
use allegro_font::{FontAddon, Font};
use allegro_primitives::PrimitivesAddon;

mod game;
mod net;

allegro_main! {

    let args = clap::App::new("client")
        .version(&crate_version!())
        .author("Ivo Wetzel <ivo.wetzel@googlemail.com>")
        .about("Server")
        .arg(clap::Arg::with_name("address:port")
            .help("Remote server address to connect to.")
            .index(1)

        ).get_matches();

    // Server address argument
    let server_addr = value_t!(
        args.value_of("address:port"), SocketAddr

    ).unwrap_or(SocketAddr::V4(
        SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 31476)
    ));

    // Setup Rendering (requires OpenGL)
    let mut core = Core::init().unwrap();
    core.set_new_display_flags(OPENGL);
    core.set_new_display_option(DisplayOption::SampleBuffers, 1, DisplayOptionImportance::Require);
    core.set_new_display_option(DisplayOption::Samples, 8, DisplayOptionImportance::Require);

    // Create display
    let disp = Display::new(&core, 768, 768).ok().expect("Failed to create OPENGL context.");
    disp.set_window_title("Rustgame: Shooter");

    // Input
    core.install_keyboard().unwrap();
    core.install_mouse().unwrap();

    // Tick / Rendering Logic
    let ticks_per_second = 30;
    let tick_dt = 1.0 / ticks_per_second as f32;
    let mut last_tick_time = 0.0;

    let frames_per_second = 60;
    let mut last_frame_time = 0.0;
    let mut frames_to_render = 0;
    let mut frame_time = 0.0;

    let mut key_state: [bool; 255] = [false; 255];
    let mut tick: i32 = 0;
    let mut redraw = false;

    // Addons
    let timer = Timer::new(&core, 1.0 / frames_per_second as f64).unwrap();
    let prim = PrimitivesAddon::init(&core).unwrap();
    let font_addon = FontAddon::init(&core).unwrap();
    let font = Font::new_builtin(&font_addon).unwrap();

    let q = EventQueue::new(&core).unwrap();
    q.register_event_source(disp.get_event_source());
    q.register_event_source(core.get_keyboard_event_source());
    q.register_event_source(core.get_mouse_event_source());
    q.register_event_source(timer.get_event_source());

    // Game instance
    let mut game = game::Game::new(&core);
    game.init(&core);

    // Network
    let mut network = net::Network::new(ticks_per_second, server_addr);
    timer.start();

    // Main Loop
    'exit: loop {

        if redraw {

            // Local Tick Logic -----------------------------------------------
            if frames_to_render == 0 {
                frames_to_render = frames_per_second / ticks_per_second;
                last_tick_time = frame_time;
                game.tick(&mut network, &key_state, tick as u8, tick_dt as f32);
                tick = (tick + 1) % 256;
            }

            // Rendering ------------------------------------------------------
            let u = 1.0 / (tick_dt as f32) * (frame_time - last_tick_time) as f32;
            let dt = frame_time - last_frame_time;

            //println!("- {}, {}", frame_time - last_tick_time, u);;
            game.draw(&core, &prim, &font, &mut network, dt as f32, u as f32);
            disp.flip();

            last_frame_time = frame_time;
            frames_to_render -= 1;
            redraw = false;

        }

        // Inputs and Events --------------------------------------------------
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

            TimerTick{timestamp: t, ..} => {
                frame_time = t;
                redraw = true;
            },

            _ => ()

        }

        // Network ------------------------------------------------------------
        while let Ok(event) = network.try_recv(frame_time) {
            match event {

                net::EventType::Connection(_) => {
                    game.connect(&core);
                },

                // Message Events come before the tick event
                net::EventType::Message(_, data) =>  {
                    // TODO set last_state_time for interpolation of remote
                    // entities. This means we need another U value.
                    game.state(&data);
                },

                net::EventType::ConnectionLost(_) => {
                    game.disconnect(&core);
                },

                _ => {}

            }
        }

    }

}

