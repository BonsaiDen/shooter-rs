#[macro_use]
extern crate clap;
extern crate rand;
extern crate shared;
extern crate shooter_server;

#[macro_use]
extern crate allegro;
extern crate allegro_sys;
extern crate allegro_font;
extern crate allegro_primitives;


// External Dependencies ------------------------------------------------------
use std::net::SocketAddr;
use shared::Lithium::Renderer;


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
    AllegroRenderer::run(
        game::Game::client(),
        game::Game::new(
            value_t!(args.value_of("address:port"), SocketAddr).ok()
        )
    );

}

