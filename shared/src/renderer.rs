use color::Color;
use particle::Particle;

pub trait Renderer {

    fn resize(&mut self, width: i32, height: i32);

    fn clear(&mut self, color: &Color);

    fn draw(&mut self, dt: f32, u: f32);

    fn triangle(
        &mut self, color: &Color,
        ax: f32, ay: f32,
        bx: f32, by: f32,
        cx: f32, cy: f32,
        line_width: f32
    );

    fn text(&mut self, color: &Color, x: f32, y: f32, text: &str);

    fn particle(&mut self) -> Option<&mut Particle>;

}
