// External Dependencies ------------------------------------------------------
use std::thread;
use std::time::Duration;
use std::net::{UdpSocket, SocketAddr};
use shooter_server::game::Game as ServerGame;


// Internal Dependencies ------------------------------------------------------
use game::{Game, ClientHandle};
use shared::{Color, ColorName};
use shared::Lithium::Server;
use self::super::{View, MenuView, GameView};


// View Implementation --------------------------------------------------------
#[derive(Debug)]
pub struct ConnectView {
    server_addr: Option<SocketAddr>
}

impl ConnectView {
    pub fn new(server_addr: Option<SocketAddr>) -> ConnectView {
        ConnectView {
            server_addr: server_addr
        }
    }
}

impl View for ConnectView {

    fn name(&self) -> &str {
        "Connect"
    }

    fn push(&mut self, game: &mut Game, client: &mut ClientHandle) {

        if self.server_addr.is_none() {

            // Get a unused, random port
            let addr = UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap();

            // Start Server
            thread::spawn(move|| {
                println!("[Client] Starting local server...");
                run_server(addr, 30);
            });

            // Ensure that the server is up and running
            thread::sleep(Duration::from_millis(20));

            self.server_addr = Some(addr);

        }

        println!("[Client] Connecting...");

        // Connect to server
        client.network.connect(self.server_addr.unwrap()).expect("Already connected!");
        game.reset(client);

    }

    fn connect(&mut self, game: &mut Game, _: &mut ClientHandle) {
        game.set_view(Box::new(GameView::new(self.server_addr.unwrap())));
    }

    fn disconnect(&mut self, game: &mut Game, _: &mut ClientHandle, _: bool) {
        // TODO implement retry when connecting to remote server
        println!("[Client] Connection failed.");
        game.set_view(Box::new(MenuView));
    }

    fn draw(&mut self, _: &mut Game, client: &mut ClientHandle) {

        // Retry Connections
        //let timeout = client.renderer.time() - game.last_connection_retry;
        //if game.state == GameState::Disconnected && timeout > 3.0 {
        //    println!("[Client] Establishing connection...");
        //    game.last_connection_retry = client.renderer.time();
        //    client.network.reset().ok();
        //}

        if let Ok(addr) = client.network.peer_addr() {
            client.renderer.text(
                &Color::from_name(ColorName::White),
                0.0, 0.0,
                &format!("Connecting to {}...", addr)[..]
            );
        }
    }

}

fn run_server(server_addr: SocketAddr, tick_rate: u32) {
    if let Err(err) = Server::run(
        server_addr,
        ServerGame::server(tick_rate, true)
    ) {
        println!("[Server] [Fatal] {:?}", err);
    }
}

