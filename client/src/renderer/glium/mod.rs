// External Dependencies ------------------------------------------------------
use rand::{SeedableRng, XorShiftRng};


// Internal Dependencies ------------------------------------------------------
mod traits;
mod particle_system;
use shared::Color;
use shared::Lithium::Renderer;
use renderer::Particle;
use self::particle_system::GliumParticleSystem;


// Glium Based Renderer -------------------------------------------------------
pub struct GliumRenderer {

    // Timing
    frame_rate: u32,
    tick_rate: u32,
    time: f64,
    dt: f32,
    u: f32,

    // Drawing
    particle_system: GliumParticleSystem,
    interpolation_ticks: usize,

    // Input
    key_state: [bool; 256],
    key_state_old: [bool; 256],

    // Internal State
    rng: XorShiftRng,

}

impl GliumRenderer {

    // Window Handling --------------------------------------------------------
    pub fn set_title(&mut self, title: &str) {

    }

    pub fn resize(&mut self, width: i32, height: i32) {

    }


    // Input ------------------------------------------------------------------
    pub fn key_down(&mut self, key_code: u8) -> bool {
        self.key_state[key_code as usize]
    }

    pub fn key_pressed(&mut self, key_code: u8) -> bool {
        self.key_state[key_code as usize] && !self.key_state_old[key_code as usize]
    }

    pub fn key_released(&mut self, key_code: u8) -> bool {
        !self.key_state[key_code as usize] && self.key_state_old[key_code as usize]
    }


    // Drawing Methods --------------------------------------------------------
    pub fn clear(&mut self, color: &Color) {
    }

    pub fn text(&mut self, color: &Color, x: f32, mut y: f32, text: &str) {
    }

    pub fn particle(&mut self) -> Option<&mut Particle> {
        self.particle_system.get()
    }

    pub fn draw_particles(&mut self) {
    }


    // RNG --------------------------------------------------------------------
    pub fn reseed_rng(&mut self, seed: [u32; 4]) {
        self.rng.reseed(seed);
    }

    pub fn rng(&mut self) -> &mut XorShiftRng {
        &mut self.rng
    }

    // Color Conversion -------------------------------------------------------
    pub fn get_color(color: &Color) -> [f32; 4] {
        [
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0
        ]
    }

}

