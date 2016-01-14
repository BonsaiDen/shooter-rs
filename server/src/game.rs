// External Dependencies ------------------------------------------------------
use std::collections::HashMap;
use cobalt::{Connection, ConnectionID, MessageKind, Handler, Server};


// Internal Dependencies ------------------------------------------------------
use shared::arena;
use shared::entities;
use shared::entity;
use shared::color::Color;
use shared::util::IdPool;


// Server Side Game Logic -----------------------------------------------------
pub struct Game {
    id_pool: IdPool<u16>,
    entities: Vec<entity::Entity>,
    arena: arena::Arena,
    available_colors: Vec<Color>,
    tick_rate: u32,
    tick: u16
}

impl Game {
    pub fn new(width: u32, height: u32, border: u32, tps: u32) -> Game {
        Game {
            id_pool: IdPool::new(),
            entities: Vec::new(),
            arena: arena::Arena::new(width, height, border),
            available_colors: Color::all_colored().into_iter().rev().collect(),
            tick_rate: tps,
            tick: 0
        }
    }
}

impl Handler<Server> for Game {

    fn bind(&mut self, _: &mut Server) {
        println!("[Server] Started");
    }

    fn connection(&mut self, _: &mut Server, conn: &mut Connection) {

        println!("[Client {}] Connected", conn.peer_addr());

        // Send Arena Configuration
        let mut config = [0].to_vec();
        config.extend(self.arena.serialize());
        conn.send(MessageKind::Reliable, config);

        // Create a ship entity from one of the available ids / colors
        if let Some(id) = self.id_pool.get_id() {

            if let Some(color) = self.available_colors.pop() {

                // TODO abstract this into some nicer shape with a factory or something
                let mut player_ship = entities::Ship::create_entity(1.0);
                let (x, y) = self.arena.center();
                let flags = color.to_flags();

                player_ship.set_state(entity::State {
                    x: x as f32,
                    y: y as f32,
                    flags: flags,
                    .. entity::State::default()
                });

                player_ship.set_id(id);
                player_ship.set_alive(true);
                player_ship.set_owner(conn.id());
                player_ship.server_created(self.tick as u8);

                self.entities.push(player_ship);

                // TODO send event? or do this via state updates only?
                // probably send player joined event but add entity via
                // state change detection on client

            }

        } else {
            unreachable!();
        }

    }

    fn tick_connections(
        &mut self, _: &mut Server,
        connections: &mut HashMap<ConnectionID, Connection>
    ) {

        let tick_dt = 1.0 / self.tick_rate as f32;

        // Tick entities
        for entity in self.entities.iter_mut() {

            // Receive inputs for entities which are controlled by clients
            if let Some(conn) = connections.get_mut(entity.owner()) {

                // TODO how to handle accelerated inputs when no remote
                // input is received
                for m in conn.received() {
                    entity.remote_input(
                        entity::Input::from_serialized(&m[..]),
                        self.tick_rate as usize
                    );
                }

            }

            // Permanently advance entity state
            entity.server_tick(&self.arena, self.tick as u8, tick_dt);

            // TODO store last N states of all entities
            // TODO perform collision detection based against
            // last confirmed client tick (aka last_input_tick)

        }

        // Send entity states to all clients
        for (_, conn) in connections.iter_mut() {

            // Calculate all entity states for the connection
            let mut states = [1, self.tick as u8].to_vec();
            for entity in self.entities.iter() {
                states.extend(entity.serialize_state(&conn.id()));
            }

            // TODO send old states for extra delay / interpolation?

            // We don't care about dropped packets
            conn.send(MessageKind::Instant, states);

        }

        // Server side tick
        self.tick = (self.tick + 1) % 256;

        // TODO bullets are handled by pre-creating a local object and then
        // syncing it with the remote one, we submit a local ID and the server
        // return this ID along with the remote object ID when updating

        // TODO server side collision is checked on each server tick
        // positions are warped to the last known local tick of the player
        // BUT there is a maximum tick difference to prevent cheating

    }

    fn connection_lost(&mut self, _: &mut Server, conn: &mut Connection) {

        println!("[Client {}] Disconnected", conn.peer_addr());

        // Find any associated entity for the connection and destroy it
        for entity in self.entities.iter_mut() {
            if entity.owned_by(&conn.id()) {

                entity.set_alive(false);
                entity.server_destroyed(self.tick as u8);

                // Release id and color
                let color = Color::from_flags(entity.get_state().flags);
                println!("[Client {}] Destroyed entity (color {:?})", conn.peer_addr(), color);

                self.id_pool.release_id(entity.id());
                self.available_colors.push(color);

            }
        }

        // Remove and dead entities from the list
        self.entities.retain(|ref entity| entity.alive());

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

