// External Dependencies ------------------------------------------------------
use std::collections::HashMap;
use cobalt::{Connection, ConnectionID, MessageKind, Handler, Server};
use lithium::{entity, event, Level};


// Internal Dependencies ------------------------------------------------------
use shared::level;
use shared::entities;
use shared::NetworkMessage;
use shared::event::Event;
use shared::color::Color;


// Server Side Game Logic -----------------------------------------------------
pub struct Game {
    manager: entity::Manager,
    events: event::Handler<Event>,
    level: level::Level,
    available_colors: Vec<Color>,
}

impl Game {
    pub fn new(width: u32, height: u32, border: u32, tick_rate: u32) -> Game {
        Game {
            manager: entity::Manager::new(
                tick_rate as u8, 1000, 75,
                true,
                Box::new(entities::Registry)
            ),
            events: event::Handler::new(),
            level: level::Level::new(width, height, border),
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

        // Send Tick / Level Configuration
        let mut config = [NetworkMessage::ServerConfig as u8].to_vec();
        config.extend(self.manager.serialize_config());
        config.extend(self.level.serialize());
        conn.send(MessageKind::Reliable, config);

        // Create a ship entity from one of the available color
        if let Some(color) = self.available_colors.pop() {

            let (x, y) = self.level.center();
            let state = entity::State {
                x: x as f32,
                y: y as f32,
                flags: color.to_flags(),
                .. entity::State::default()
            };

            self.manager.create_entity(0, Some(state), Some(&conn.id()));

            // TODO support entity events?
            // TODO send event? or do this via state updates only?
            // probably send player joined event but add entity via
            // state change detection on client

            self.events.send(Event::PlayerJoined);

        }

    }

    fn tick_connections(
        &mut self, _: &mut Server,
        connections: &mut HashMap<ConnectionID, Connection>
    ) {

        let tick_dt = 1.0 / self.manager.config().tick_rate as f32;

        // Receive Data
        for (id, conn) in connections.iter_mut() {
            for data in conn.received() {
                match NetworkMessage::from_u8(data[0]) {
                    NetworkMessage::ClientInput => {

                        // Extract all unconfirmed inputs the client sent us
                        if let Some(entity) = self.manager.get_entity_for_owner(id) {
                            let data = &data[1..];
                            for i in data.chunks(entity::Input::encoded_size()) {
                                entity.remote_input(
                                    entity::Input::from_serialized(i)
                                );
                            }
                        }

                    },
                    NetworkMessage::ClientEvents => {
                        self.events.receive_events(*id, &data[1..]);
                    },
                    _=> println!("Unknown Client Message {:?}", data)
                }
            }
        }

        // Tick Entities
        self.manager.tick_entities(
            &self.level,
            tick_dt,
            |_, _, _, _| {

            }, |_, _, _, _| {

            }
        );

        // Send Data
        let events = self.events.serialize_events();
        for (id, conn) in connections.iter_mut() {

            // Send entity states to all clients (We don't care about dropped packets)
            let mut data = [NetworkMessage::ServerState as u8].to_vec();
            data.extend(self.manager.serialize_state(id));
            conn.send(MessageKind::Instant, data);

            // Send events to all clients (Make sure the arrive eventually)
            if let Some(ref events) = events {
                let mut data = [NetworkMessage::ServerEvents as u8].to_vec();
                data.extend(events.clone());
                conn.send(MessageKind::Reliable, data);
            }

        }

        // TODO bullets are handled by pre-creating a local object and then
        // syncing it with the remote one, we submit a local ID and the server
        // return this ID along with the remote object ID when updating

        // TODO server side collision is checked on each server tick
        // positions are warped to the last known local tick of the player
        // BUT there is a maximum tick difference to prevent cheating

    }

    fn connection_lost(&mut self, _: &mut Server, conn: &mut Connection) {

        println!("[Client {}] Disconnected", conn.peer_addr());

        // Find the associated entity for the connection and destroy it
        if let Some(id) = self.manager.get_entity_id_for_owner(&conn.id()) {
            if let Some(entity) = self.manager.destroy_entity(id) {
                let color = Color::from_flags(entity.state().flags);
                println!("[Client {}] Destroyed entity ({:?})", conn.peer_addr(), color);
                self.available_colors.push(color);
            }
        };

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

