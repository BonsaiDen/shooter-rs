use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use cobalt::{Client, Connection, ConnectionID, MessageKind, Handler, Server};

use shared::arena;
use shared::entities;
use shared::entity::{Entity, EntityInput, EntityState};
use shared::color::Color;


// Server Side Game Logic -----------------------------------------------------
pub struct Game {
    entities: Vec<Entity>,
    arena: arena::Arena,
    available_colors: Vec<Color>

}

impl Game {
    pub fn new(width: u32, height: u32, border: u32) -> Game {
        Game {
            entities: Vec::new(),
            arena: arena::Arena::new(width, height, border),
            available_colors: Color::all_colored()
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

        // Create a ship entity from one of the available colors
        if let Some(color) = self.available_colors.pop() {

            let mut player_ship = entities::ship::Ship(1.0);
            let (x, y) = self.arena.center();

            let flags = color.to_flags();
            println!("{} flags", flags);
            player_ship.typ.set_state(EntityState {
                x: x as f32,
                y: y as f32,
                flags: flags,
                .. EntityState::default()
            });

            player_ship.set_alive(true);
            player_ship.set_owner(conn.id());

            self.entities.push(player_ship);

            println!("[Client {}] Created entity (color {:?})", conn.peer_addr(), color);

            // TODO send event? or do this via state updates only?
            // probably send player joined event but add entity via update

        }

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

        println!("[Client {}] Disconnected", conn.peer_addr());

        // Find any associated entity for the connection and destroy it
        for entity in self.entities.iter_mut() {
            if entity.owned_by(&conn.id()) {

                let color = Color::from_flags(entity.typ.get_state().flags);
                entity.set_alive(false);
                entity.typ.destroy();

                // Make color available again
                self.available_colors.push(color);

                println!("[Client {}] Destroyed entity (color {:?})", conn.peer_addr(), color);

            }
        }

        // Remove and dead entities from the list
        self.entities.retain(|ref entity| entity.alive());

        // TODO have actual disconnect command?

    }

}

