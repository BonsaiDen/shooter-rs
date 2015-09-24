#[macro_use]
extern crate clap;
extern crate rand;
extern crate cobalt;
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
    let disp = Display::new(&core, 600, 600).ok().expect("Failed to create OPENGL context.");
    disp.set_window_title("Rustgame: Shooter");

    // Input
    core.install_keyboard().unwrap();
    core.install_mouse().unwrap();

    // Addons
    let timer = Timer::new(&core, 1.0 / 60.0).unwrap();
    let prim = PrimitivesAddon::init(&core).unwrap();
    let font_addon = FontAddon::init(&core).unwrap();
    let font = Font::new_builtin(&font_addon).unwrap();

    let q = EventQueue::new(&core).unwrap();
    q.register_event_source(disp.get_event_source());
    q.register_event_source(core.get_keyboard_event_source());
    q.register_event_source(core.get_mouse_event_source());
    q.register_event_source(timer.get_event_source());
    timer.start();

    // Tick / Rendering Logic
    let ticks_per_second = 30;
    let tick_dt = 1.0 / (ticks_per_second as f32);

    let mut key_state: [bool; 255] = [false; 255];
    let mut tick: i32 = 0;
    let mut last_tick_time: f64 = 0.0;

    let mut redraw = true;
    let mut last_frame_time: f64 = 0.0;
    let mut last_render_time: f64 = 0.0;
    let mut last_u = 0.0;
    let mut render_time: f64 = 0.0;

    // Network
    let mut network = net::Network::new(ticks_per_second, server_addr);

    // Game instance
    let mut game = game::Game::new(&core);
    game.init(&core);

    // Main Loop
    'exit: loop {

        // Rendering
        if redraw {
            let dt = render_time - last_render_time;
            let mut u = 1.0 / (tick_dt as f64) * (render_time - last_tick_time);

            // Smooth out render stutter from interference in out delta time
            if u < 0.9 && last_u < 0.9 {
                u *= last_u;
            }
            last_u = u;

            game.draw(&core, &prim, &font, &mut network, dt as f32, u as f32);
            disp.flip();
            redraw = false;
        }

        // Network Events
        while let Ok(event) = network.try_recv(last_tick_time) {
            match event {

                net::EventType::Tick => {
                    last_tick_time = last_frame_time;
                    game.tick(&mut network, &key_state, tick as u8, tick_dt);
                    tick = (tick + 1) % 256;
                },

                net::EventType::Message(_, _) =>  {
                    // TODO decode packet data and create events more events?
                },

                _ => {
                    println!("Received a message from network: {:?}", event);
                }

            }
        }

        // Grab all pending Input Events
        // This reduces render stutter
        while q.is_empty() == false {
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
                    last_frame_time = t;
                    last_render_time = render_time;
                    render_time = t;
                    redraw = true;
                },

                _ => () // println!("Some other event...")

            }
        }

    }

}

