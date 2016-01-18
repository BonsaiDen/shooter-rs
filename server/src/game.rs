// External Dependencies ------------------------------------------------------
use std::collections::HashMap;
use cobalt::{Connection, ConnectionID, MessageKind, Handler, Server};


// Internal Dependencies ------------------------------------------------------
use lithium::{entity, IdPool, Level};

use shared::level;
use shared::entities;
use shared::color::Color;


// Server Side Game Logic -----------------------------------------------------
pub struct Game {
    id_pool: IdPool<u16>,
    entities: Vec<entity::Entity>,
    level: level::Level,
    available_colors: Vec<Color>,
    interpolation_ticks: u8,
    tick_rate: u8,
    tick: u16
}

impl Game {
    pub fn new(width: u32, height: u32, border: u32, tps: u32) -> Game {
        Game {
            id_pool: IdPool::new(),
            entities: Vec::new(),
            level: level::Level::new(width, height, border),
            available_colors: Color::all_colored().into_iter().rev().collect(),
            interpolation_ticks: 3,
            tick_rate: tps as u8,
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

        // Send Tick / Level Configuration
        let mut config = [
            0,
            self.tick_rate,
            self.interpolation_ticks as u8,
            30

        ].to_vec();

        config.extend(self.level.serialize());

        conn.send(MessageKind::Reliable, config);

        // Create a ship entity from one of the available ids / colors
        if let Some(id) = self.id_pool.get_id() {

            if let Some(color) = self.available_colors.pop() {

                let (x, y) = self.level.center();

                let mut player_ship = entities::Ship::create_entity(1.0);
                player_ship.set_state(entity::State {
                    x: x as f32,
                    y: y as f32,
                    flags: color.to_flags(),
                    .. entity::State::default()
                });

                player_ship.set_id(id);
                player_ship.set_alive(true);
                player_ship.set_owner(conn.id());
                player_ship.event(entity::Event::Created(self.tick as u8));

                self.entities.push(player_ship);

                // TODO support entity events?
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

            // Check if the entity is controlled by a client
            let owner_connection = if let Some(owner) = entity.owner() {
                connections.get_mut(&*owner)

            } else {
                None
            };

            // Receive input messages for controlled clients
            if let Some(conn) = owner_connection {

                for m in conn.received() {

                    // Extract all unconfirmed inputs the client sent us
                    for i in m.chunks(entity::Input::encoded_size()) {
                        entity.remote_input(
                            entity::Input::from_serialized(i)
                        );
                    }
                }

            }

            // Permanently advance entity state
            entity.server_tick(&self.level, self.tick as u8, tick_dt);

            // TODO perform collision detection based against
            // last confirmed client tick (aka remote_input_tick)

        }

        // Send entity states to all clients
        for (id, conn) in connections.iter_mut() {

            // Calculate all entity states for the connection
            // TODO do we need the server tick value?
            let mut states = [1, self.tick as u8, 0].to_vec();
            for entity in self.entities.iter() {

                // Confirm latest input tick from the client
                if entity.owned_by(id) {
                    states[2] = entity.confirmed_tick();
                }

                states.extend(entity.serialize_state(&conn.id()));

            }

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
                entity.event(entity::Event::Destroyed(self.tick as u8));

                // Release id and color
                let color = Color::from_flags(entity.state().flags);
                println!("[Client {}] Destroyed entity ({:?})", conn.peer_addr(), color);

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

