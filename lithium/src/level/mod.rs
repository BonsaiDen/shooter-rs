// External Dependencies ------------------------------------------------------
use std::ops::{Deref, DerefMut};


// Internal Dependencies ------------------------------------------------------
use entity::State;
use renderer::Renderer;

mod traits;
pub use level::traits::Base as Base;
pub use level::traits::Drawable as Drawable;


// Level Wrapper Structure ----------------------------------------------------
pub struct Level<S: State, L: Base<S>> {
    level: L,
    drawable: Box<Drawable<S>>,
}

impl<S: State, L: Base<S>> Level<S, L> {

    pub fn new(level: L, drawable: Box<Drawable<S>>) -> Level<S, L> {
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
        self.drawable.draw(renderer, &self.level);
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.level.serialize()
    }

}


// Dereference to access internal level logic ---------------------------------
impl<S: State, L: Base<S>> Deref for Level<S, L> {
    type Target = L;
    fn deref(&self) -> &L {
        &self.level
    }
}

impl<S: State, L: Base<S>> DerefMut for Level<S, L> {
    fn deref_mut(&mut self) -> &mut L {
        &mut self.level
    }
}

