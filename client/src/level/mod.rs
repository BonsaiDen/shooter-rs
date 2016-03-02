// External Dependencies ------------------------------------------------------
use lithium::{level, renderer};
use lithium::level::Base as LithiumLevelBase;


// Internal Dependencies ------------------------------------------------------
use shared::level::Level;
use shared::state::State;


// Level Drawable Implementation Dependencies ----------------------------------
pub struct DrawableLevel;
impl DrawableLevel {

    pub fn from_serialized(data: &[u8]) -> level::Level<State, Level> {
        level::Level::new(
            Level::from_serialized(data),
            Box::new(DrawableLevel)
        )
    }

}

impl level::Drawable<State> for DrawableLevel {

    fn draw(&mut self, _: &mut renderer::Renderer, _: &level::Base<State>) {

    }

}

