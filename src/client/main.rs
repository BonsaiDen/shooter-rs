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
use std::thread;
use std::cmp;
use std::time::Duration;

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

    // Addons
    let prim = PrimitivesAddon::init(&core).unwrap();
    let font_addon = FontAddon::init(&core).unwrap();
    let font = Font::new_builtin(&font_addon).unwrap();

    let mut q = EventQueue::new(&core).unwrap();
    q.register_event_source(disp.get_event_source());
    q.register_event_source(core.get_keyboard_event_source());
    q.register_event_source(core.get_mouse_event_source());

    // Tick / Rendering Logic
    let ticks_per_second = 30;
    let tick_dt = 1000000000 / ticks_per_second;
    let mut last_tick_time: u64 = clock_ticks::precise_time_ns();

    let frames_per_second = 60;
    let frame_dt = 1000000000 / frames_per_second;
    let mut last_frame_time: u64 = last_tick_time - frame_dt;

    let mut key_state: [bool; 255] = [false; 255];
    let mut tick: i32 = 0;

    // Game instance
    let mut game = game::Game::new(&core);
    game.init(&core);

    // Network
    let mut network = net::Network::new(ticks_per_second, server_addr);

    // Main Loop
    'exit: loop {

        let frame_time = clock_ticks::precise_time_ns();

        // Inputs -------------------------------------------------------------
        while let Some(e) = q.next() {
            match e {

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

                _ => ()

            }
        }

        // Network ------------------------------------------------------------
        while let Ok(event) = network.try_recv(frame_time / 1000000) {
            match event {

                net::EventType::Connection(_) => {
                    game.connect(&core);
                },

                // Message Events come before the tick event
                net::EventType::Message(_, data) =>  {
                    game.state(&data);
                },

                net::EventType::Tick => {
                    last_tick_time = frame_time;
                    game.tick(&mut network, &key_state, true, tick as u8, tick_dt as f32 * 0.000000001);
                    tick = (tick + 1) % 256;
                },

                net::EventType::ConnectionLost(_) => {
                    game.disconnect(&core);
                },

                _ => {
                    println!("Received a message from network: {:?}", event);
                }

            }
        }

        // Rendering ----------------------------------------------------------
        let u = 1.0 / (tick_dt as f32) * (frame_time - last_tick_time) as f32;
        let dt = frame_time - last_frame_time;

        //println!("- {}, {}", frame_time - last_tick_time, u);;
        if dt != 0 {
            game.draw(&core, &prim, &font, &mut network, dt as f32 * 0.000000001, u as f32);
            disp.flip();
            last_frame_time = frame_time;
        }

        let diff = cmp::min(clock_ticks::precise_time_ns() - frame_time, frame_dt);
        thread::sleep(Duration::new(0, (frame_dt - diff) as u32));

    }

}

