use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use cobalt::{Client, Connection, ConnectionID, MessageKind, Handler, Server};

use shared::arena;
use shared::entity::{Entity, EntityInput, EntityState, EntityItem};


// Server Side Game Logic -----------------------------------------------------
pub struct Game {
    entities: Vec<Box<Entity>>,
    arena: arena::Arena
}

impl Game {
    pub fn new(width: u32, height: u32, border: u32) -> Game {
        Game {
            entities: Vec::new(),
            arena: arena::Arena::new(width, height, border)
        }
    }
}

impl Handler<Server> for Game {

    fn bind(&mut self, _: &mut Server) {
        println!("[Server] Started");
    }

    fn tick_connections(
        &mut self, _: &mut Server,
        connections: &mut HashMap<ConnectionID, Connection>
    ) {

        // Receive inputs
        for (_, conn) in connections.iter_mut() {
            // Apply all new inputs and set confirmed client tick
        }

        // TODO Ticks all entities
            // TODO store last N states of all entities
            // TODO send delta based off of last confirmed client tick
            // TODO perform collision detection based against last confirmed client tick

        // TODO Send state to all clients
        for (_, conn) in connections.iter_mut() {

        }

        // TODO bullets are handled by pre-creating a local object and then
        // syncing it with the remote one, we submit a local ID and the server
        // return this ID along with the remote object ID when updating

        // TODO server side collision is checked on each server tick
        // positions are warped to the last known local tick of the player
        // BUT there is a maximum tick difference to prevent cheating

    }

    fn shutdown(&mut self, _: &mut Server) {
        println!("Server::shutdown");
    }

    fn connection(&mut self, _: &mut Server, conn: &mut Connection) {

        println!("[Client {}] Connected", conn.peer_addr());

        // Send Arena Configuration
        let mut config = [0].to_vec();
        config.extend(self.arena.serialize());
        conn.send(MessageKind::Reliable, config);

        // TODO create ship entity from one of the available colors

    }

    fn connection_packet_lost(
        &mut self, _: &mut Server, _: &mut Connection, p: &[u8]
    ) {
        println!("Server::connection_packet_loss {}", p.len());
    }

    fn connection_congestion_state(&mut self, _: &mut Server, _: &mut Connection, state: bool) {
        println!("Server::connection_congestion_state {}", state);
    }

    fn connection_lost(&mut self, _: &mut Server, conn: &mut Connection) {
        // TODO destroy ship entity and make color available again
        println!("[Client {}] Disconnected", conn.peer_addr());
    }

}

