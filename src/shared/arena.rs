use entity::EntityState;

pub struct Arena {
    width: u32,
    height: u32,
    border: u32
}

impl Arena {

    pub fn new(width: u32, height: u32, border: u32) -> Arena {
        Arena {
            width: width,
            height: height,
            border: border
        }
    }

    pub fn wrap_state(&self, state: &mut EntityState) {

        let width = (self.height + self.border) as f32;
        if state.x < 0.0 {
            state.x += width;

        } else if state.x >= width {
            state.x -= width;
        }

        let height = (self.height + self.border) as f32;
        if state.y < 0.0 {
            state.y += height;

        } else if state.y >= height {
            state.y -= height;
        }

    }

    pub fn interpolate_state(
        &self, current: &EntityState, last: &EntityState, u: f32

    ) -> EntityState {

        // Skip interpolation if distance is too large too avoid glitching
        // when wrapping at the arena boundaries occurs
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
        EntityState {
            r: last.r + mr.sin().atan2(mr.cos()) * u,
            x: x,
            y: y,
            mx: last.mx,
            my: last.my,
            flags: current.flags
        }

    }

}

