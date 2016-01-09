use std::collections::HashMap;
use cobalt::{Connection, ConnectionID, MessageKind, Handler, Server};

use shared::arena;
use shared::entities;
use shared::entity::{Entity, EntityInput, EntityState};
use shared::color::Color;
use shared::util::IdPool;


// Server Side Game Logic -----------------------------------------------------
pub struct Game {
    entities: Vec<Entity>,
    arena: arena::Arena,
    available_colors: Vec<Color>,
    id_pool: IdPool<u16>,
    tick: u16
}

impl Game {
    pub fn new(width: u32, height: u32, border: u32) -> Game {
        Game {
            entities: Vec::new(),
            arena: arena::Arena::new(width, height, border),
            available_colors: Color::all_colored().into_iter().rev().collect(),
            id_pool: IdPool::new(),
            tick: 0
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

        // TODO configure on server and send with config to client?
        let ticks_per_second = 30;
        let tick_dt = 1.0 / ticks_per_second as f32;

        // Tick all entities
        for entity in self.entities.iter_mut() {

            // Receive inputs for the entity
            if let Some(conn) = connections.get_mut(entity.owner()) {

                // Apply all new inputs and set confirmed client tick
                for m in conn.received() {
                    entity.typ.input(EntityInput::from_serialized(&m[..]));
                }

            }

            // Tick entity
            entity.typ.tick(&self.arena, tick_dt);

            // And mark the state as confirmed (TODO clean this up)
            let state = entity.typ.get_state();
            entity.typ.remote_tick(&self.arena, tick_dt, self.tick as u8, state);

            // TODO store last N states of all entities
            // TODO perform collision detection based against last confirmed client tick (aka
            // last_input_tick)?
        }


        // Send entity states to all clients
        for (_, conn) in connections.iter_mut() {

            // Calculate entity states for all connections
            let mut state = [1, self.tick as u8].to_vec();
            for entity in self.entities.iter() {
                state.extend(entity.serialize(&conn.id()));
            }
            conn.send(MessageKind::Instant, state);

        }

        self.tick = (self.tick + 1) % 256;

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
        if let Some(id) = self.id_pool.get_id() {

            if let Some(color) = self.available_colors.pop() {

                let mut player_ship = entities::Ship::create_entity(1.0);
                let (x, y) = self.arena.center();

                let flags = color.to_flags();
                player_ship.set_state(EntityState {
                    x: x as f32,
                    y: y as f32,
                    flags: flags,
                    .. EntityState::default()
                });

                player_ship.set_id(id);
                player_ship.set_alive(true);
                player_ship.set_owner(conn.id());

                println!("set owner {:?}", conn.id() == *player_ship.owner());

                self.entities.push(player_ship);

                println!("[Client {}] Created entity (color {:?})", conn.peer_addr(), color);

                // TODO send event? or do this via state updates only?
                // probably send player joined event but add entity via
                // state change detection on client

            }

        } else {
            unreachable!();
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

                let color = Color::from_flags(entity.get_state().flags);
                entity.set_alive(false);
                entity.destroy();
                self.id_pool.release_id(entity.id());

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

