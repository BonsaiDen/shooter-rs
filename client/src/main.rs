#[macro_use]
extern crate clap;
extern crate rand;
extern crate shared;
extern crate shooter_server;

#[cfg(feature="allegro_renderer")]
#[macro_use]
extern crate allegro;
#[cfg(feature="allegro_renderer")]
extern crate allegro_sys;
#[cfg(feature="allegro_renderer")]
extern crate allegro_font;
#[cfg(feature="allegro_renderer")]
extern crate allegro_primitives;


// External Dependencies ------------------------------------------------------
use std::net::SocketAddr;
use shared::Lithium::Renderer as LithiumRenderer;


// Internal Dependencies ------------------------------------------------------
mod entities;
mod game;
mod level;
mod renderer;
use renderer::Renderer;


// Main -----------------------------------------------------------------------
#[cfg(feature="allegro_renderer")]
allegro_main! { run(); }

fn run() {

    let args = clap::App::new("shooter-client")
        .version(&crate_version!())
        .author("Ivo Wetzel <ivo.wetzel@googlemail.com>")
        .about("Shooter-Client")
        .arg(clap::Arg::with_name("address:port")
            .help("Remote server address to connect to.")
            .index(1)

        ).get_matches();


    // Arguments --------------------------------------------------------------
    Renderer::run(game::Game::client(
        value_t!(args.value_of("address:port"), SocketAddr).ok()
    ));

}
