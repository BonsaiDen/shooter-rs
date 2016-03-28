// External Dependencies ------------------------------------------------------
use std::thread;
use std::time::Duration;
use std::net::{UdpSocket, SocketAddr};
use shooter_server::game::Game as ServerGame;


// Internal Dependencies ------------------------------------------------------
use game::{Game, ClientHandle};
use shared::{Color, ColorName};
use shared::Lithium::{Server, Renderer};
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

    fn push(&mut self, game: &mut Game, handle: &mut ClientHandle) {

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
        handle.client.connect(self.server_addr.unwrap()).expect("Already connected!");
        game.reset(handle);

    }

    fn connect(&mut self, game: &mut Game, _: &mut ClientHandle) {
        game.set_view(Box::new(GameView::new(self.server_addr.unwrap())));
    }

    fn disconnect(&mut self, _: &mut Game, handle: &mut ClientHandle, _: bool) {
        println!("[Client] Connection failed, trying again in 3 seconds...");
        handle.timer.schedule(Box::new(|_, handle| {
            println!("[Client] Retrying connection...");
            handle.client.reset().ok();

        }), 3000);
    }

    fn draw(&mut self, game: &mut Game, handle: &mut ClientHandle) {

        handle.renderer.clear(&Color::from_name(ColorName::Black));

        if let Ok(addr) = handle.client.peer_addr() {
            handle.renderer.text(
                &Color::from_name(ColorName::White),
                0.0, 0.0,
                &format!("Connecting to {}... (press ESC to cancel)", addr)[..]
            );
        }

        if handle.renderer.key_released(59) {
            handle.client.close().ok();
            game.set_view(Box::new(MenuView));
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

