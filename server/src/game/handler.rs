// External Dependencies ------------------------------------------------------
use shared::Lithium::Cobalt::{Connection, ConnectionID, ConnectionMap};
use shared::Lithium::{Entity, EntityState, Level, DefaultRenderer,
                      ServerHandle, ServerHandler};


// Internal Dependencies ------------------------------------------------------
use game::Game;
use shared::{Color, SharedEvent, SharedCommand, SharedLevel, SharedState};


// Type Aliases ---------------------------------------------------------------
pub type Handle<'a> = ServerHandle<'a, SharedEvent, SharedState, SharedLevel, DefaultRenderer>;
pub type ServerLevel = Level<SharedState, SharedLevel>;
pub type ServerEntity = Entity<SharedState, SharedLevel, DefaultRenderer>;


// Handler Implementation -----------------------------------------------------
impl ServerHandler<SharedEvent, SharedState, SharedLevel, DefaultRenderer> for Game {

    fn bind(&mut self, _: Handle) {
        println!("[Server] Started");
    }

    fn connect(&mut self, _: Handle, conn: &mut Connection) {
        println!("[Server] [Client {}] Connected", conn.peer_addr());
    }

    fn disconnect(&mut self, handle: Handle, conn: &mut Connection) {

        println!("[Server] [Client {}] Disconnected", conn.peer_addr());

        while let Some(id) = handle.entities.get_entity_id_for_owner(&conn.id()) {
            if let Some(entity) = handle.entities.destroy(id) {
                let color = Color::from_flags(entity.state().flags);
                println!("[Client {}] Destroyed entity ({:?})", conn.peer_addr(), color);
                self.available_colors.push(color);
            }
        }

    }

    fn event(
        &mut self, handle: Handle, _: &mut ConnectionMap, owner: ConnectionID, event: SharedEvent
    ) {

        println!("[Server] [Client {:?}] Event: {:?}", owner, event);

        match event {
            SharedEvent::JoinGame => {

                if let Some(_) = handle.entities.get_entity_for_owner(&owner) {
                    println!("[Server] [Client {:?}] Already has a entity.", owner);

                } else {

                    // Create a ship entity from one of the available colors
                    if let Some(color) = self.available_colors.pop() {

                        let (x, y) = handle.level.center();
                        let state = SharedState {
                            x: x as f32,
                            y: y as f32,
                            flags: color.to_flags(),
                            .. Default::default() // TODO implement default trait
                        };

                        handle.entities.create(
                            0,
                            Some(state),
                            Some(&owner)
                        );

                        handle.events.send(Some(owner), SharedEvent::GameJoined);
                        handle.events.send(None, SharedEvent::PlayerJoined);

                    }

                }
            },
            SharedEvent::Command(SharedCommand::Shutdown) if self.loopback_mode => {
                println!("[Server] [Client {:?}] Received Shutdown Command", owner);
                handle.server.shutdown().unwrap();
            },
            _ => { println!("Unknown Event") }
        }

    }

    fn tick_before(&mut self, _: Handle, _: &mut ConnectionMap) {

        // TODO bullets are handled by pre-creating a local object and then
        // syncing it with the remote one, we submit a local ID and the server
        // return this ID along with the remote object ID when updating

        // TODO server side collision is checked on each server tick
        // positions are warped to the last known local tick of the player
        // BUT there is a maximum tick difference to prevent cheating

    }

    fn tick_entity_before(
        &mut self, _: &ServerLevel, _: &mut ServerEntity, _: u8, _: f32
    ) {

    }

    fn tick_entity_after(
        &mut self, _: &ServerLevel, _: &mut ServerEntity, _: u8, _: f32
    ) {

    }

    fn tick_after(&mut self, _: Handle, _: &mut ConnectionMap) {

    }

    fn shutdown(&mut self, _: Handle) {
        println!("[Server] Shutdown");
    }

}

