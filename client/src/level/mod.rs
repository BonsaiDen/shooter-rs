// External Dependencies ------------------------------------------------------
use lithium;
use lithium::level::Base as LithiumLevelBase;


// Internal Dependencies ------------------------------------------------------
use shared::level::Level;
use shared::state::State;


// Level Drawable Implementation Dependencies ----------------------------------
pub struct DrawableLevel;
impl DrawableLevel {

    //pub fn create(width: u32, height: u32, border: u32) -> lithium::level::Level<State, level::Level> {
    //    lithium::level::Level::new(
    //        level::Level::new(width, height, border),
    //        Box::new(Level)
    //    )
    //}

    pub fn from_serialized(data: &[u8]) -> lithium::level::Level<State, Level> {
        lithium::level::Level::new(
            Level::from_serialized(data),
            Box::new(DrawableLevel)
        )
    }

}

impl lithium::level::Drawable<State> for DrawableLevel {

    fn draw(&mut self, _: &mut lithium::renderer::Renderer, _: &lithium::level::Base<State>) {

    }

}

