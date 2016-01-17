#[macro_use]
extern crate clap;
extern crate rand;
extern crate cobalt;
extern crate lithium;
extern crate shared;

#[macro_use]
extern crate allegro;
extern crate allegro_sys;
extern crate allegro_font;
extern crate allegro_primitives;

mod entities;
mod game;
mod net;
mod renderer;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use lithium::Renderer;
use renderer::AllegroRenderer;

// Main -----------------------------------------------------------------------
allegro_main! {

    let args = clap::App::new("client")
        .version(&crate_version!())
        .author("Ivo Wetzel <ivo.wetzel@googlemail.com>")
        .about("Client")
        .arg(clap::Arg::with_name("address:port")
            .help("Remote server address to connect to.")
            .index(1)

        ).get_matches();


    // Arguments --------------------------------------------------------------
    let server_addr = value_t!(
        args.value_of("address:port"), SocketAddr

    ).unwrap_or(SocketAddr::V4(
        SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 31476)
    ));


    // Client Setup -----------------------------------------------------------
    AllegroRenderer::run(game::Game::new(server_addr));

}

