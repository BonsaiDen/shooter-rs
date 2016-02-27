// External Dependencies ------------------------------------------------------
use std::collections::HashMap;
use cobalt::{Connection, ConnectionID, Handler, Server};
use lithium::{entity, Level, Server as LithiumServer};


// Internal Dependencies ------------------------------------------------------
use shared::level;
use shared::entities;
use shared::event::Event;
use shared::color::Color;


// Server Side Game Logic -----------------------------------------------------
pub struct Game {
    server: LithiumServer<Event, level::Level>,
    available_colors: Vec<Color>
}

impl Game {
    pub fn new(width: u32, height: u32, border: u32, tick_rate: u32) -> Game {
        Game {
            server: LithiumServer::new(
                tick_rate, 1000, 75,
                level::Level::new(width, height, border),
                Box::new(entities::Registry)
            ),
            available_colors: Color::all_colored().into_iter().rev().collect(),
        }
    }
}

impl Handler<Server> for Game {

    fn bind(&mut self, _: &mut Server) {
        println!("[Server] Started");
    }

    fn connection(&mut self, _: &mut Server, conn: &mut Connection) {

        println!("[Client {}] Connected", conn.peer_addr());

        self.server.init_connection(conn);

        // Create a ship entity from one of the available color
        if let Some(color) = self.available_colors.pop() {

            let (x, y) = self.server.level().center();
            let state = entity::State {
                x: x as f32,
                y: y as f32,
                flags: color.to_flags(),
                .. entity::State::default()
            };

            self.server.entities().create_entity(
                0, Some(state), Some(&conn.id())
            );
            self.server.events().send(Event::PlayerJoined);

        }

    }

    fn tick_connections(
        &mut self, _: &mut Server,
        connections: &mut HashMap<ConnectionID, Connection>
    ) {

        self.server.tick(connections, |_, _, _, _| {

        }, |_, _, _, _| {

        })

        // TODO bullets are handled by pre-creating a local object and then
        // syncing it with the remote one, we submit a local ID and the server
        // return this ID along with the remote object ID when updating

        // TODO server side collision is checked on each server tick
        // positions are warped to the last known local tick of the player
        // BUT there is a maximum tick difference to prevent cheating

    }

    fn connection_lost(&mut self, _: &mut Server, conn: &mut Connection) {

        println!("[Client {}] Disconnected", conn.peer_addr());

        let available_colors = &mut self.available_colors;
        self.server.close_connection(conn, |conn, entity| {
            let color = Color::from_flags(entity.state().flags);
            println!("[Client {}] Destroyed entity ({:?})", conn.peer_addr(), color);
            available_colors.push(color);
        })

    }

    fn connection_packet_lost(
        &mut self, _: &mut Server, _: &mut Connection, p: &[u8]
    ) {
        println!("Server::connection_packet_loss {}", p.len());
    }

    fn connection_congestion_state(&mut self, _: &mut Server, _: &mut Connection, state: bool) {
        println!("Server::connection_congestion_state {}", state);
    }

    fn shutdown(&mut self, _: &mut Server) {
        println!("Server::shutdown");
    }

}

