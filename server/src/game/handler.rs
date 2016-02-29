// External Dependencies ------------------------------------------------------
use std::collections::HashMap;
use lithium::entity;
use lithium::level;
use lithium::server::{Handler, Handle};
use cobalt::{Connection, ConnectionID};


// Internal Dependencies ------------------------------------------------------
use game::Game;
use shared::color::Color;
use shared::event::Event;
use shared::level::Level;
use shared::state::State;


// Handler Implementation -----------------------------------------------------
impl Handler<Event,  State> for Game {

    fn bind(&mut self, _: Handle<Event,  State>) {
        println!("[Server] Started");
    }

    fn connect(
        &mut self,
        _: Handle<Event, State>,
        conn: &mut Connection
    ) {
        println!("[Client {}] Connected", conn.peer_addr());
    }

    fn disconnect(
        &mut self,
        server: Handle<Event,  State>,
        conn: &mut Connection
    ) {

        println!("[Client {}] Disconnected", conn.peer_addr());

        while let Some(id) = server.entities.get_entity_id_for_owner(&conn.id()) {
            if let Some(entity) = server.entities.destroy(id) {
                let color = Color::from_flags(entity.state().flags);
                println!("[Client {}] Destroyed entity ({:?})", conn.peer_addr(), color);
                self.available_colors.push(color);
            }
        }

    }

    fn event(
        &mut self,
        server: Handle<Event,  State>,
        owner: ConnectionID,
        event: Event
    ) {

        println!("[Client {:?}] Event: {:?}", owner, event);

        match event {
            Event::JoinGame => {

                if let Some(_) = server.entities.get_entity_for_owner(&owner) {
                    println!("[Client {:?}] Already has a entity.", owner);

                } else {

                    // Create a ship entity from one of the available colors
                    if let Some(color) = self.available_colors.pop() {

                        let level = Level::downcast_mut(server.level);
                        let (x, y) = level.center();
                        let state = State {
                            x: x as f32,
                            y: y as f32,
                            flags: color.to_flags(),
                            .. entity::State::default()
                        };

                        server.entities.create(
                            0,
                            Some(state),
                            Some(&owner)
                        );

                        server.events.send(Some(owner), Event::GameJoined);
                        server.events.send(None, Event::PlayerJoined);

                    }

                }
            },
            _ => {}
        }

    }

    fn tick_before(
        &mut self,
        _: Handle<Event,  State>,
        _: &mut HashMap<ConnectionID, Connection>,
        _: u8, _: f32
    ) {

        // TODO bullets are handled by pre-creating a local object and then
        // syncing it with the remote one, we submit a local ID and the server
        // return this ID along with the remote object ID when updating

        // TODO server side collision is checked on each server tick
        // positions are warped to the last known local tick of the player
        // BUT there is a maximum tick difference to prevent cheating

    }

    fn tick_entity_before(
        &mut self,
        _: &level::Level<State>,
        _: &mut entity::Entity<State>,
        _: u8, _: f32
    ) {

    }

    fn tick_entity_after(
        &mut self,
        _: &level::Level<State>,
        _: &mut entity::Entity<State>,
        _: u8, _: f32
    ) {

    }

    fn tick_after(
        &mut self,
        _: Handle<Event, State>,
        _: &mut HashMap<ConnectionID, Connection>,
        _: u8, _: f32
    ) {

    }

    fn shutdown(&mut self, _: Handle<Event, State>) {
        println!("[Server] Shutdown");
    }

}
