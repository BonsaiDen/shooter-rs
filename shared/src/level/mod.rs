// External Dependencies ------------------------------------------------------
use lithium::{Level, DrawableLevel};


// Internal Dependencies ------------------------------------------------------
use state::SharedState;
mod traits;


// Shared Level Logic ---------------------------------------------------------
#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct SharedLevel {
    width: u32,
    height: u32,
    border: u32
}

impl SharedLevel {

    pub fn new(width: u32, height: u32, border: u32) -> SharedLevel {
        SharedLevel {
            width: width,
            height: height,
            border: border
        }
    }

    pub fn create(
        width: u32, height: u32, border: u32

    ) -> Level<SharedState, SharedLevel> {
        Level::new(
            SharedLevel {
                width: width,
                height: height,
                border: border
            },
            Box::new(NoneDrawable)
        )
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn center(&self) -> (u32, u32) {
        (self.width / 2 + self.border, self.height / 2 + self.border)
    }

}


// Noop Drawable --------------------------------------------------------------
struct NoneDrawable;
impl DrawableLevel<SharedState> for NoneDrawable {}

