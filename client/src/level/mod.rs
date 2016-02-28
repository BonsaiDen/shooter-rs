// External Dependencies ------------------------------------------------------
use lithium;
use lithium::level::Base as LithiumLevelBase;


// Internal Dependencies ------------------------------------------------------
use shared::level;
use shared::state;


// Level Drawable Implementation Dependencies ----------------------------------
pub struct Level;

impl Level {

    pub fn create(width: u32, height: u32, border: u32) -> lithium::level::Level<state::State> {
        lithium::level::Level::new(
            Box::new(level::Level::new(width, height, border)),
            Box::new(Level)
        )
    }

    pub fn from_serialized(data: &[u8]) -> lithium::level::Level<state::State> {
        lithium::level::Level::new(
            Box::new(level::Level::from_serialized(data)),
            Box::new(Level)
        )
    }

}

impl lithium::level::Drawable<state::State> for Level {

    fn draw(&mut self, _: &mut lithium::renderer::Renderer, _: &lithium::level::Base<state::State>) {

    }

}

