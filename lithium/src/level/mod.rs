// External Dependencies ------------------------------------------------------
use std::any::Any;


// Internal Dependencies ------------------------------------------------------
use entity::State;
use renderer::Renderer;

mod traits;
pub use level::traits::Base as Base;
pub use level::traits::Drawable as Drawable;


// Level Wrapper Structure ----------------------------------------------------
pub struct Level<S: State> {
    level: Box<Base<S>>,
    drawable: Box<Drawable<S>>,
}

impl<S: State> Level<S> {

    pub fn new(level: Box<Base<S>>, drawable: Box<Drawable<S>>) -> Level<S> {
        Level {
            level: level,
            drawable: drawable
        }
    }

    pub fn limit_state(&self, state: &mut S) {
        self.level.limit_state(state);
    }

    pub fn interpolate_entity_state(
        &self,
        renderer: &mut Renderer,
        current: &S, last: &S

    ) -> S {
        self.level.interpolate_entity_state(renderer, current, last)
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        self.drawable.draw(renderer, &*self.level);
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.level.serialize()
    }

    pub fn as_any(&mut self) -> &mut Any {
        self.level.as_any()
    }

}

