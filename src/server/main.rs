#[macro_use]
extern crate clap;
extern crate cobalt;
extern crate shared;

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use cobalt::{Client, Config, Connection, ConnectionID, MessageKind, Handler, Server};

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

    let ticks_per_second = 30;
    let config = Config {
        send_rate: 30,
        .. Config::default()
    };
    let mut handler = ServerHandler;
    let mut server = Server::new(config);
    server.bind(&mut handler, server_addr).unwrap();

}

struct ServerHandler;
impl Handler<Server> for ServerHandler {

    fn bind(&mut self, _: &mut Server) {
        println!("Server::bind");
    }

    fn tick_connections(
        &mut self, _: &mut Server,
        connections: &mut HashMap<ConnectionID, Connection>
    ) {
        for (_, conn) in connections.iter_mut() {

        }
    }

    fn shutdown(&mut self, _: &mut Server) {
        println!("Server::shutdown");
    }

    fn connection(&mut self, _: &mut Server, _: &mut Connection) {
        println!("Server::connection");
    }

    fn connection_failed(&mut self, _: &mut Server, _: &mut Connection) {
        println!("Server::connection_failed");
    }

    fn connection_packet_lost(
        &mut self, _: &mut Server, _: &mut Connection, p: &[u8]
    ) {
        println!("Server::connection_packet_loss {}", p.len());
    }

    fn connection_congestion_state(&mut self, _: &mut Server, _: &mut Connection, state: bool) {
        println!("Server::connection_congestion_state {}", state);
    }

    fn connection_lost(&mut self, _: &mut Server, _: &mut Connection) {
        println!("Server::connection_lost");
    }

}

