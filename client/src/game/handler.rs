// Internal Dependencies ------------------------------------------------------
use shared::Lithium::ClientHandler;
use shared::Lithium::Cobalt::ConnectionID;
use renderer::AllegroRenderer;
use shared::{SharedEvent, SharedLevel, SharedState};
use game::{Game, ClientHandle, ClientEntity, ClientLevel};


// Macros ---------------------------------------------------------------------
macro_rules! with_view_state {
    ($s:ident, $v:ident, $c:ident, $b:block) => ({
        if let Some(mut $v) = $s.view.take() {

            $b;

            if let Some(mut next) = $s.next_view.take() {
                $v.pop($s, &mut $c);
                println!("[View] {} was popped", $v.name());
                next.push($s, &mut $c);
                println!("[View] {} was pushed", next.name());
                $s.view = Some(next);

            } else {
                $s.view = Some($v);
            }

        }
    })
}

macro_rules! with_view {
    ($s:ident, $v:ident, $b:block) => ({
        if let Some(mut $v) = $s.view.take() {
            $b;
            $s.view = Some($v);
        }
    })
}


// Handler Implementation -----------------------------------------------------
impl ClientHandler<SharedEvent, SharedState, SharedLevel, AllegroRenderer> for Game {

    fn init(&mut self, mut client: ClientHandle) {
        with_view_state!(self, view, client, {
            view.init(self, &mut client);
        });
    }

    fn config(&mut self, mut client: ClientHandle, level_data: &[u8]) {
        with_view_state!(self, view, client, {
            view.config(self, &mut client, level_data);
        });
    }

    fn connect(&mut self, mut client: ClientHandle) {
        with_view_state!(self, view, client, {
            view.connect(self, &mut client);
        });
    }

    fn disconnect(&mut self, mut client: ClientHandle, was_connected: bool) {
        with_view_state!(self, view, client, {
            view.disconnect(self, &mut client, was_connected);
        });
    }

    fn event(&mut self, mut client: ClientHandle, owner: ConnectionID, event: SharedEvent) {
        with_view_state!(self, view, client, {
            view.event(self, &mut client, owner, event);
        });
    }

    fn tick_before(&mut self, mut client: ClientHandle) {
        with_view_state!(self, view, client, {
            view.tick_before(self, &mut client);
        });
    }

    fn tick_entity_before(
        &mut self,
        renderer: &mut AllegroRenderer,
        level: &ClientLevel,
        entity: &mut ClientEntity,
        tick: u8, dt: f32
    ) {
        with_view!(self, view, {
            view.tick_entity_before(self, renderer, level, entity, tick, dt);
        });
    }

    fn tick_entity_after(
        &mut self,
        renderer: &mut AllegroRenderer,
        level: &ClientLevel,
        entity: &mut ClientEntity,
        tick: u8, dt: f32
    ) {
        with_view!(self, view, {
            view.tick_entity_after(self, renderer, level, entity, tick, dt);
        });
    }

    fn tick_after(&mut self, mut client: ClientHandle) {
        with_view_state!(self, view, client, {
            view.tick_after(self, &mut client);
        });
    }

    fn draw(&mut self, mut client: ClientHandle) {
        with_view_state!(self, view, client, {
            view.draw(self, &mut client);
        });
    }

    fn destroy(&mut self, mut client: ClientHandle) {
        with_view_state!(self, view, client, {
            view.destroy(self, &mut client);
        });
    }

}

