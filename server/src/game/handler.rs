// External Dependencies ------------------------------------------------------
use shared::Lithium::Cobalt::{Connection, ConnectionID, ConnectionMap};
use shared::Lithium::{DefaultRenderer, ServerHandler};


// Internal Dependencies ------------------------------------------------------
use game::{Game, ServerHandle, ServerLevel, ServerEntity};
use shared::{Color, SharedEvent, SharedCommand, SharedLevel, SharedState, SharedRegistry};


// Handler Implementation -----------------------------------------------------
impl ServerHandler<DefaultRenderer, SharedRegistry, SharedLevel, SharedEvent, SharedState> for Game {

    fn bind(&mut self, handle: ServerHandle) {
        println!("[Server] Started");
        self.count(handle);
    }

    fn connect(&mut self, _: ServerHandle, conn: &mut Connection) {
        println!("[Server] [Client {}] Connected", conn.peer_addr());
    }

    fn disconnect(&mut self, handle: ServerHandle, conn: &mut Connection) {
        println!("[Server] [Client {}] Disconnected", conn.peer_addr());
        self.disconnect_client(handle, conn);
    }

    fn event(
        &mut self, handle: ServerHandle, connections: &mut ConnectionMap,
        owner: ConnectionID, event: SharedEvent
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

                        handle.events.send_to(Some(owner), SharedEvent::GameJoined);
                        handle.events.send(SharedEvent::PlayerJoined);

                    }

                }
            },

            SharedEvent::LeaveGame => {
                let mut conn = connections.get_mut(&owner).unwrap();
                self.disconnect_client(handle, conn);
                conn.close(); // TODO speed up detection for connection drop
            },

            SharedEvent::Command(SharedCommand::Shutdown) if self.loopback_mode => {
                println!("[Server] [Client {:?}] Received Shutdown Command", owner);
                handle.server.shutdown().unwrap();
            },
            _ => { println!("[Server] Unknown Event") }
        }

    }

    fn tick_before(&mut self, _: ServerHandle, _: &mut ConnectionMap) {

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

    fn tick_after(&mut self, _: ServerHandle, _: &mut ConnectionMap) {

    }

    fn shutdown(&mut self, _: ServerHandle) {
        println!("[Server] Shutdown");
    }

}

impl Game {

    pub fn disconnect_client(&mut self, handle: ServerHandle, conn: &mut Connection) {
        while let Some(id) = handle.entities.get_entity_id_for_owner(&conn.id()) {
            if let Some(entity) = handle.entities.destroy(id) {
                let color = Color::from_flags(entity.state().flags);
                println!("[Server] [Client {}] Destroyed entity ({:?})", conn.peer_addr(), color);
                self.available_colors.push(color);
            }
        }
    }

}

