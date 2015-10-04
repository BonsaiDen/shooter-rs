#[macro_use]
extern crate clap;
extern crate cobalt;
extern crate shared;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use cobalt::{Config, Handler, Server};

mod game;

fn main() {

    let args = clap::App::new("server")
        .version(&crate_version!())
        .author("Ivo Wetzel <ivo.wetzel@googlemail.com>")
        .about("Server")
        .arg(clap::Arg::with_name("address:port")
            .help("Local server address to bind to.")
            .index(1)

        ).get_matches();

    // Server address argument
    let server_addr = value_t!(
        args.value_of("address:port"), SocketAddr

    ).unwrap_or(SocketAddr::V4(
        SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 31476)
    ));

    let mut game = game::Game::new(512, 512, 16);
    let mut server = Server::new(Config {
        send_rate: 30,
        .. Config::default()
    });
    server.bind(&mut game, server_addr).unwrap();

}

