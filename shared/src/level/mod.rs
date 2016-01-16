// Internal Dependencies ------------------------------------------------------
use entity;
use std::f32;


// Internal Dependencies ------------------------------------------------------
use renderer::Renderer;


// Level Abstractions ---------------------------------------------------------
pub struct Level {
    width: u32,
    height: u32,
    border: u32
}

impl Level {

    // Constructors -----------------------------------------------------------
    pub fn new(width: u32, height: u32, border: u32) -> Level {
        Level {
            width: width,
            height: height,
            border: border
        }
    }

    pub fn from_serialized(data: &[u8]) -> Level {
        Level {
            width: ((data[0] as u32) << 8) | data[1] as u32,
            height: ((data[2] as u32) << 8) | data[3] as u32,
            border: data[4] as u32
        }
    }


    // Static Methods ---------------------------------------------------------
    pub fn limit_state(&self, state: &mut entity::State) {

        let width = (self.height + self.border * 2) as f32;
        if state.x < 0.0 {
            state.x += width;

        } else if state.x >= width {
            state.x -= width;
        }

        let height = (self.height + self.border * 2) as f32;
        if state.y < 0.0 {
            state.y += height;

        } else if state.y >= height {
            state.y -= height;
        }

    }

    pub fn interpolate_entity_state(
        &self,
        renderer: &mut Renderer,
        current: &entity::State, last: &entity::State

    ) -> entity::State {

        let u = renderer.delta_u();

        // Skip interpolation if distance is too large too avoid glitching
        // when wrapping at the level boundaries occurs
        let x = if (last.x - current.x).abs() < self.width as f32 * 0.5 {
            last.x * (1.0 - u) + current.x * u

        } else {
            current.x
        };

        let y = if (last.y - current.y).abs() < self.height as f32 * 0.5 {
            last.y * (1.0 - u) + current.y * u

        } else {
            current.y
        };

        let mr = current.r - last.r;
        entity::State {
            r: (last.r + mr.sin().atan2(mr.cos()) * u) % (f32::consts::PI * 2.0),
            x: x - self.border as f32,
            y: y - self.border as f32,
            mx: last.mx,
            my: last.my,
            flags: current.flags
        }

    }

    // Getters ----------------------------------------------------------------
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn center(&self) -> (u32, u32) {
        (self.width / 2 + self.border, self.height / 2 + self.border)
    }

    pub fn serialize(&self) -> Vec<u8> {
        [
            (self.width >> 8) as u8,
            self.width as u8,
            (self.height >> 8) as u8,
            self.height as u8,
            self.border as u8

        ].to_vec()
    }

}

