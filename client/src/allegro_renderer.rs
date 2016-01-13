use allegro::{Core, Display, Color as AllegroColor};
use allegro_font::{FontDrawing, FontAddon, Font, FontAlign};
use allegro_primitives::PrimitivesAddon;


use shared::color::Color;
use shared::renderer::Renderer;
use shared::particle::{Particle, ParticleSystem};


pub struct AllegroRenderer {
    core: Core,
    display: Display,
    prim: PrimitivesAddon,
    font: Font,
    particle_system: ParticleSystem
}

impl AllegroRenderer {
    pub fn new(core: Core, display: Display) -> AllegroRenderer {

        let prim = PrimitivesAddon::init(&core).unwrap();
        let font_addon = FontAddon::init(&core).unwrap();
        let font = Font::new_builtin(&font_addon).unwrap();

        AllegroRenderer {
            core: core,
            display: display,
            prim: prim,
            font: font,
            particle_system: ParticleSystem::new(1000)
        }

    }
}

impl Renderer for AllegroRenderer {

    fn resize(&mut self, width: i32, height: i32) {
        self.display.resize(width, height).ok();
    }

    fn clear(&mut self, color: &Color) {
        self.core.clear_to_color(get_color(color));
    }

    fn draw(&mut self, dt: f32, u: f32) {
        let prim = &self.prim;
        self.particle_system.draw(dt, |ref color, s, x, y| {
            prim.draw_filled_rectangle(
                x - s + 0.5, y - s + 0.5, x + s + 0.5, y + s + 0.5,
                get_color(color)
            );
        });
        self.core.flip_display();
    }

    fn triangle(
        &mut self, color: &Color,
        ax: f32, ay: f32,
        bx: f32, by: f32,
        cx: f32, cy: f32,
        line_width: f32
    ) {
        self.prim.draw_triangle(ax, ay, bx, by, cx, cy, get_color(color), line_width);
    }

    fn text(&mut self, color: &Color, x: f32, y: f32, text: &str) {
        self.core.draw_text(&self.font, get_color(color), x, y, FontAlign::Left, text);
    }

    fn particle(&mut self) -> Option<&mut Particle> {
        self.particle_system.get()
    }

}

fn get_color(color: &Color) -> AllegroColor {
    let a = color.a as f32 / 255.0;
    AllegroColor::from_rgb(
        (color.r as f32 * a) as u8,
        (color.g as f32 * a) as u8,
        (color.b as f32 * a) as u8
    )
}

