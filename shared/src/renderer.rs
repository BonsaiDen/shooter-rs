use color::Color;
use particle::Particle;

pub trait Renderer {

    fn set_fps(&mut self, frame_rate: u32);

    fn get_fps(&mut self) -> u32;

    fn set_time(&mut self, time: f64);

    fn get_time(&mut self) -> f64;

    fn set_tick_rate(&mut self, tick_rate: u32);

    fn get_tick_rate(&mut self) -> u32;

    fn set_delta_time(&mut self, dt: f32);

    fn set_delta_u(&mut self, u: f32);

    fn get_delta_time(&mut self) -> f32;

    fn get_delta_u(&mut self) -> f32;

    fn set_title(&mut self, title: &str);

    fn do_draw(&mut self) -> bool;

    fn events(&mut self);

    fn running(&mut self) -> bool;

    fn key_down(&mut self, key_code: u8) -> bool;

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
