#[macro_use]
extern crate clap;
extern crate rand;
extern crate cobalt;
extern crate lithium;
extern crate shared;
extern crate shooter_server;

#[macro_use]
extern crate allegro;
extern crate allegro_sys;
extern crate allegro_font;
extern crate allegro_primitives;


// External Dependencies ------------------------------------------------------
use std::thread;
use std::str::FromStr;
use std::net::SocketAddr;
use std::time::Duration;
use lithium::{Renderer, Server};


// Internal Dependencies ------------------------------------------------------
use renderer::AllegroRenderer;
mod entities;
mod game;
mod level;
mod renderer;


// Main -----------------------------------------------------------------------
allegro_main! {

    let args = clap::App::new("shooter-client")
        .version(&crate_version!())
        .author("Ivo Wetzel <ivo.wetzel@googlemail.com>")
        .about("Shooter-Client")
        .arg(clap::Arg::with_name("address:port")
            .help("Remote server address to connect to.")
            .index(1)

        ).get_matches();


    // Arguments --------------------------------------------------------------
    if let Ok(remote_addr) = value_t!(
        args.value_of("address:port"), SocketAddr
    ) {
        run_client(remote_addr);

    } else {

        let local_addr = SocketAddr::from_str("127.0.0.1:31475").unwrap();
        let server_thread = thread::spawn(move|| {
            run_server(local_addr, 30);
        });

        // Ensure that the server is up and running
        // TODO make the reconnect timeout configurable
        thread::sleep(Duration::from_millis(10));

        run_client(local_addr);
        server_thread.join().unwrap();

    };

}

fn run_client(server_addr: SocketAddr) {
    AllegroRenderer::run(
        game::Game::client(server_addr),
        game::Game::new()
    );
}

fn run_server(server_addr: SocketAddr, tick_rate: u32) {
    Server::run(
        server_addr,
        shooter_server::game::Game::server(tick_rate, true)
    );
}

