#[macro_use]
extern crate clap;
extern crate shared;


// External Dependencies ------------------------------------------------------
use std::str::FromStr;
use std::net::SocketAddr;
use shared::Lithium::Server;


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

    ).unwrap_or(SocketAddr::from_str("127.0.0.1:31475").unwrap());


    // Server Setup -----------------------------------------------------------
    if let Err(err) = Server::run(server_addr, game::Game::server(30, false)) {
        println!("[Server] [Fatal] {:?}", err);
    }

}

