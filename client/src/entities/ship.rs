// External Dependencies ------------------------------------------------------
use rand::Rng;
use std::{cmp, f32};


// Internal Dependencies ------------------------------------------------------
use shared::entities::Ship;
use shared::{SharedState, SharedLevel, Color, ColorName};
use shared::Lithium::{Entity, EntityState, EntityEvent, DrawableEntity, Level};
use renderer::AllegroRenderer;


// Ship Drawable Implementation Dependencies ----------------------------------
pub struct RenderedShip {
    color_light: Color,
    color_mid: Color,
    scale: f32,
    particle_count: u32,
    last_draw_state: SharedState
}

impl RenderedShip {

    pub fn create_entity(scale: f32) -> Entity<SharedState, SharedLevel, AllegroRenderer> {
        Entity::new(
            Box::new(Ship::new(scale)),
            Box::new(RenderedShip::new(scale))
        )
    }

    pub fn new(scale: f32) -> RenderedShip {
        RenderedShip {
            color_light: Color::from_name(ColorName::Grey),
            color_mid: Color::from_name(ColorName::Grey).darken(0.5),
            scale: scale,
            particle_count: 5,
            last_draw_state: Default::default()
        }
    }

}

impl DrawableEntity<SharedState, SharedLevel, AllegroRenderer> for RenderedShip {

    fn event(&mut self, event: &EntityEvent, _: &SharedState) {
        if let &EntityEvent::Flags(flags) = event {
            self.color_light = Color::from_flags(flags);
            self.color_mid = self.color_light.darken(0.5);
        }
    }

    fn draw(
        &mut self,
        renderer: &mut AllegroRenderer,
        _: &Level<SharedState, SharedLevel>,
        state: SharedState
    ) {

        let light = &self.color_light;
        let mid = &self.color_mid;
        let scale = self.scale;

        self.last_draw_state.set_to(&state);

        // Rendering
        draw_triangle(
            renderer, &state,
            mid, scale, scale, 1.15, -9.0, 6.0
        );
        draw_triangle(
            renderer, &state,
            light, scale, scale, (2 as f32).sqrt(), 12.0, 9.0
        );
        draw_triangle(
            renderer, &state,
            mid, scale, scale * 0.66, (2 as f32).sqrt(), 12.0, 9.0
        );

        // Effects
        if state.flags & 0x02 == 0x02 {

            if renderer.rng().gen::<u8>() > 50 || self.particle_count > 1 {

                // Exhause more particles initially
                for _ in 0..self.particle_count {

                    let r = renderer.rng().gen::<u8>() as f32;
                    let v = renderer.rng().gen::<u8>() as f32;

                    if let Some(p) = renderer.particle() {

                        // Exhaust angle
                        let w = 0.95;
                        let mr = state.my.atan2(state.mx);
                        let d = state.r - mr;

                        // Increase engine velocity when flying backwards
                        let mut dr = d.abs() % (f32::consts::PI * 2.0);
                        if dr > f32::consts::PI  {
                            dr = f32::consts::PI * 2.0 - dr;
                        }

                        // Calculate exhaust angle
                        let cs = (1.0 - w) * mr.cos() + w * state.r.cos();
                        let sn = (1.0 - w) * mr.sin() + w * state.r.sin();
                        let mr = sn.atan2(cs) + f32::consts::PI;
                        let ar = (r / 255.0 - 0.5) * (f32::consts::PI * 0.65);

                        // Spawn exhaust particles
                        p.color.set_to(&self.color_light);
                        p.x = state.x + mr.cos() * 9.0 * self.scale + 0.5;
                        p.y = state.y + mr.sin() * 9.0 * self.scale + 0.5;
                        p.s = 2.5 * self.scale;
                        p.sms = -1.25 * self.scale;
                        p.v = ((86.0 + v / 9.0) * 0.5 + dr * 30.0) * 0.5 * self.scale;
                        p.vms = 0.0;
                        p.r = mr - ar * 1.7;

                        // Spread out exhaust
                        p.rms = ar * 1.25;

                        p.fadeout = 0.35;
                        p.lifetime = 0.4;
                        p.remaining = p.lifetime;

                    }

                }

            }

            self.particle_count = 1;

        } else {
            self.particle_count = cmp::min(self.particle_count + 1, 5);
        }

    }

}


// Helpers --------------------------------------------------------------------
fn draw_triangle(
    renderer: &mut AllegroRenderer,
    state: &SharedState, color: &Color,
    base_scale: f32, body_scale: f32, dr: f32, da: f32, db: f32
) {
    let beta = f32::consts::PI / dr;
    let ox = state.r.cos() * -2.0 * base_scale + 0.5;
    let oy = state.r.sin() * -2.0 * base_scale + 0.5;
    let ax = ox + state.x + state.r.cos() * da * body_scale;
    let ay = oy + state.y + state.r.sin() * da * body_scale;
    let bx = ox + state.x + (state.r + beta).cos() * db * body_scale;
    let by = oy + state.y + (state.r + beta).sin() * db * body_scale;
    let cx = ox + state.x + (state.r - beta).cos() * db * body_scale;
    let cy = oy + state.y + (state.r - beta).sin() * db * body_scale;
    renderer.triangle(
        color, ax, ay, bx, by, cx, cy, 0.5 * body_scale
    );
}

