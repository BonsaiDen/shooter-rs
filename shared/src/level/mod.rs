// External Dependencies ------------------------------------------------------
use lithium;


// Internal Dependencies ------------------------------------------------------
use state;
mod traits;


// Shared Level Logic ---------------------------------------------------------
#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct Level {
    width: u32,
    height: u32,
    border: u32
}

impl Level {

    pub fn new(width: u32, height: u32, border: u32) -> Level {
        Level {
            width: width,
            height: height,
            border: border
        }
    }

    pub fn create(width: u32, height: u32, border: u32) -> lithium::level::Level<state::State> {
        lithium::level::Level::new(
            Box::new(Level {
                width: width,
                height: height,
                border: border
            }),
            Box::new(ZeroDrawable)
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

    pub fn downcast_mut<'a>(level: &'a mut lithium::level::Level<state::State>) -> &'a mut Level {
        match level.as_any().downcast_mut::<Level>() {
            Some(r) => r,
            None => unreachable!()
        }
    }

}


// Noop Drawable --------------------------------------------------------------
struct ZeroDrawable;
impl lithium::level::Drawable<state::State> for ZeroDrawable {}

