// External Dependencies ------------------------------------------------------
use lithium::{Level, BaseLevel, DrawableLevel, Renderer};


// Internal Dependencies ------------------------------------------------------
use shared::{SharedLevel, SharedState};


// Level Drawable Implementation Dependencies ----------------------------------
pub struct RenderedLevel;
impl RenderedLevel {

    pub fn from_serialized(data: &[u8]) -> Level<SharedState, SharedLevel> {
        Level::new(
            SharedLevel::from_serialized(data),
            Box::new(RenderedLevel)
        )
    }

}

impl DrawableLevel<SharedState> for RenderedLevel {
    fn draw(&mut self, _: &mut Renderer, _: &BaseLevel<SharedState>) {
    }
}

