// Internal Dependencies ------------------------------------------------------
mod traits;


// Shared Level Abstraction ---------------------------------------------------
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

