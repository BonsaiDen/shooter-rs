#[macro_use]
extern crate clap;
extern crate cobalt;
extern crate lithium;
extern crate shared;


// External Dependencies ------------------------------------------------------
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use lithium::Server;


// Internal Dependencies ------------------------------------------------------
mod game;


// Main -----------------------------------------------------------------------
fn main() {

    let args = clap::App::new("shooter-server")
        .version(&crate_version!())
        .author("Ivo Wetzel <ivo.wetzel@googlemail.com>")
        .about("Shooter Server")
        .arg(clap::Arg::with_name("address:port")
            .help("Local server address to bind to.")
            .index(1)

        ).get_matches();


    // Arguments --------------------------------------------------------------
    let server_addr = value_t!(
        args.value_of("address:port"), SocketAddr

    ).unwrap_or(SocketAddr::V4(
        SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 31476)
    ));


    // Server Setup -----------------------------------------------------------
    Server::run(server_addr, game::Game::server(30));

}

