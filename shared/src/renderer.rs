// External Dependencies ------------------------------------------------------
use rand::XorShiftRng;


// Internal Dependencies ------------------------------------------------------
use color::Color;
use particle::Particle;

pub trait Runnable {
    fn init(&mut self, &mut Renderer);
    fn tick(&mut self, &mut Renderer) -> bool;
    fn draw(&mut self, &mut Renderer);
    fn destroy(&mut self);
}


// Renderer Abstraction -------------------------------------------------------
pub trait Renderer {

    fn run<R: Runnable>(runnable: R) where Self: Sized;

    // Time Related -----------------------------------------------------------
    fn time(&self) -> f64;

    fn set_time(&mut self, time: f64);

    fn delta_time(&self) -> f32;

    fn set_delta_time(&mut self, dt: f32);

    fn delta_u(&self) -> f32;

    fn set_delta_u(&mut self, u: f32);


    // Frame / Tick Rate ------------------------------------------------------
    fn fps(&self) -> u32;

    fn set_fps(&mut self, frame_rate: u32);

    fn tick_rate(&self) -> u32;

    fn set_tick_rate(&mut self, tick_rate: u32);


    // Interpolation ----------------------------------------------------------
    fn interpolation_ticks(&self) -> usize;

    fn set_interpolation_ticks(&mut self, ticks: usize);


    // Input ------------------------------------------------------------------
    fn key_down(&mut self, key_code: u8) -> bool;


    // RNG --------------------------------------------------------------------
    fn reseed_rng(&mut self, seed: [u32; 4]);

    fn rng(&mut self) -> &mut XorShiftRng;


    // Window -----------------------------------------------------------------
    fn set_title(&mut self, title: &str);

    fn resize(&mut self, width: i32, height: i32);


    // Drawing ----------------------------------------------------------------
    fn clear(&mut self, color: &Color);

    fn draw(&mut self);

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
