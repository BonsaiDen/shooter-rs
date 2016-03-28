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

    fn init(&mut self, mut handle: ClientHandle) {
        with_view_state!(self, view, handle, {
            view.init(self, &mut handle);
        });
    }

    fn config(&mut self, mut handle: ClientHandle, level_data: &[u8]) {
        with_view_state!(self, view, handle, {
            view.config(self, &mut handle, level_data);
        });
    }

    fn connect(&mut self, mut handle: ClientHandle) {
        with_view_state!(self, view, handle, {
            view.connect(self, &mut handle);
        });
    }

    fn disconnect(&mut self, mut handle: ClientHandle, was_connected: bool) {
        with_view_state!(self, view, handle, {
            view.disconnect(self, &mut handle, was_connected);
        });
    }

    fn event(&mut self, mut handle: ClientHandle, owner: ConnectionID, event: SharedEvent) {
        with_view_state!(self, view, handle, {
            view.event(self, &mut handle, owner, event);
        });
    }

    fn tick_before(&mut self, mut handle: ClientHandle) {
        with_view_state!(self, view, handle, {
            view.tick_before(self, &mut handle);
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

    fn tick_after(&mut self, mut handle: ClientHandle) {
        with_view_state!(self, view, handle, {
            view.tick_after(self, &mut handle);
        });
    }

    fn draw(&mut self, mut handle: ClientHandle) {
        with_view_state!(self, view, handle, {
            view.draw(self, &mut handle);
        });
    }

    fn destroy(&mut self, mut handle: ClientHandle) {
        with_view_state!(self, view, handle, {
            view.destroy(self, &mut handle);
        });
    }

}

