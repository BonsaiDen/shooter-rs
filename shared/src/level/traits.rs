// External Dependencies ------------------------------------------------------
use std::f32;
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};


// Internal Dependencies ------------------------------------------------------
use lithium::{entity, Renderer, Level as LithiumLevel};
use super::Level;


// Level Trait Implementation -------------------------------------------------
impl LithiumLevel for Level {

    fn limit_state(&self, state: &mut entity::State) {

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

    fn interpolate_entity_state(
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

    fn encoded_size() -> usize {
        12
    }

    fn from_serialized(data: &[u8]) -> Self {
        decode::<Self>(data).unwrap()
    }

    fn serialize(&self) -> Vec<u8> {
        encode(&self, SizeLimit::Infinite).unwrap()
    }

}

