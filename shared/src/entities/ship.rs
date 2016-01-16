// External Dependencies ------------------------------------------------------
use std::f32;


// Internal Dependencies ------------------------------------------------------
use entity;
use level::Level;


// Ship Logic Implementation --------------------------------------------------
pub struct Ship {
    max_speed: f32,
    acceleration: f32,
    rotation: f32
}

impl Ship {

    pub fn create_entity(scale: f32) -> entity::Entity {
        entity::Entity::new(
            Box::new(Ship::new(scale)),
            Box::new(ZeroDrawable)
        )
    }

    pub fn new(scale: f32) -> Ship {
        Ship {
            max_speed: 90.0 * scale,
            acceleration: 2.0 * scale,
            rotation: 120.0,
        }
    }

}


// Trait Implementations ------------------------------------------------------
impl entity::traits::Base for Ship {

    fn typ(&self) -> u8 {
        0
    }

    fn apply_input(
        &mut self,
        level: &Level, state: &mut entity::State, input: &entity::Input, dt: f32
    ) {

        let mut steer = 0.0;
        if input.left {
            steer -= 1.0;
        }

        if input.right {
            steer += 1.0;
        }

        // TODO make this constant time rotation
        state.r += f32::consts::PI / 180.0 * self.rotation * dt * steer;

        if input.thrust {
            // Constant time acceleration
            let m = 60.0 / (1.0 / dt);
            state.mx += state.r.cos() * self.acceleration * dt * m;
            state.my += state.r.sin() * self.acceleration * dt * m;
            state.flags |= 0x02;

        } else {
            state.flags &= !0x02;
        }

        // Limit diagonal speed
        let mr = state.my.atan2(state.mx);
        let mut s = ((state.mx * state.mx) + (state.my * state.my)).sqrt();

        // Allow for easier full stop
        if s < 0.15 {
            s *= 0.95;
        }

        // Limit max speed
        state.mx = mr.cos() * s.min(self.max_speed * dt);
        state.my = mr.sin() * s.min(self.max_speed * dt);
        state.x += state.mx;
        state.y += state.my;

        // Apply level restrictions
        level.limit_state(state);

    }

}


// Noop Drawable --------------------------------------------------------------
struct ZeroDrawable;
impl entity::traits::Drawable for ZeroDrawable {}

