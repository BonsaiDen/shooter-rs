// External Dependencies ------------------------------------------------------
use std::thread;
use std::time::Duration;
use rand::{SeedableRng, XorShiftRng};
use glium::backend::glutin_backend::GlutinFacade;
use glium::{glutin, Surface, Frame};
use clock_ticks;


// Internal Dependencies ------------------------------------------------------
mod traits;
mod particle_system;
mod font;
use shared::Color;
use shared::Lithium::Renderer;
use renderer::Particle;
use self::particle_system::GliumParticleSystem;
use self::font::Font;


// Key Code Mapping -----------------------------------------------------------
#[derive(Copy, Clone)]
pub enum KeyCode {
    A = 38,
    C = 54,
    D = 40,
    W = 25,
    Up = 111,
    Left = 113,
    Right = 114,
    LCtrl = 37,
    Enter = 36,
    Escape = 9
}


// Glium Based Renderer -------------------------------------------------------
pub struct GliumRenderer {

    // Timing
    frame_rate: u32,
    tick_rate: u32,
    time: f64,
    dt: f32,
    u: f32,

    // Drawing
    target: Option<Frame>,
    display: GlutinFacade,
    font: Font,
    perspective: [[f32; 4]; 4],
    particle_system: GliumParticleSystem,
    interpolation_ticks: usize,

    // Input
    key_state: [bool; 256],
    key_state_old: [bool; 256],

    // Internal State
    rng: XorShiftRng,
    is_running: bool

}

impl GliumRenderer {

    pub fn new(display: GlutinFacade, width: u32, height: u32) -> GliumRenderer {

        let font = Font::new(&display, "font.fnt", "font_0.png");
        let particle_system = GliumParticleSystem::new(&display, 1000);

        GliumRenderer {

            // Timing
            frame_rate: 60,
            tick_rate: 60,
            time: 0.0,
            dt: 0.0,
            u: 0.0,

            // Drawing
            target: None,
            display: display,
            font: font,
            perspective: GliumRenderer::perspective(width as f32, height as f32),

            particle_system: particle_system,
            interpolation_ticks: 0,

            // Input
            key_state: [false; 256],
            key_state_old: [false; 256],

            // Internal State
            rng: XorShiftRng::new_unseeded(),
            is_running: true

        }

    }

    // Window Handling --------------------------------------------------------
    pub fn set_title(&mut self, title: &str) {
        self.display.get_window().unwrap().set_title(title);
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.perspective = GliumRenderer::perspective(width as f32, height as f32);
        self.display.get_window().unwrap().set_inner_size(width as u32, height as u32);
    }


    // Input ------------------------------------------------------------------
    pub fn key_down(&mut self, key_code: KeyCode) -> bool {
        self.key_state[key_code as usize]
    }

    pub fn key_pressed(&mut self, key_code: KeyCode) -> bool {
        self.key_state[key_code as usize] && !self.key_state_old[key_code as usize]
    }

    pub fn key_released(&mut self, key_code: KeyCode) -> bool {
        !self.key_state[key_code as usize] && self.key_state_old[key_code as usize]
    }


    // Drawing Methods --------------------------------------------------------
    pub fn clear(&mut self, color: &Color) {
        self.target.as_mut().unwrap().clear_color(
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0
        );
    }

    pub fn text(&mut self, color: &Color, x: f32, y: f32, text: &str) {
        let mut target = self.target.as_mut().unwrap();
        self.font.draw(
            &mut target,
            &self.perspective,
            text,
            x, y,
            GliumRenderer::get_color(color)
        );
    }

    pub fn particle(&mut self) -> Option<&mut Particle> {
        self.particle_system.get()
    }

    pub fn draw_particles(&mut self) {
        let mut target = self.target.as_mut().unwrap();
        self.particle_system.draw(&mut target, &self.perspective, self.dt);
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


    // Perspective Conversion -------------------------------------------------
    fn perspective(width: f32, height: f32) -> [[f32; 4]; 4] {

        let (r, l, t, b, f, n) = (
            width as f32, 0.0, 0.0, height as f32, 100.0, 0.0
        );

        [
            [2.0 / (r - l), 0.0, 0.0, 0.0],
            [0.0, 2.0 / (t - b), 0.0, 0.0],
            [0.0, 0.0, -2.0 / (f - n), 0.0],
            [-1.0, 1.0, -(f + n) / (f - n), 1.0f32]
        ]

    }

}


// Internal Methods required for trait implementation -------------------------
impl GliumRenderer {

    fn should_draw(&mut self) -> bool {
        thread::sleep(Duration::from_millis(1000 / self.frame_rate as u64));
        self.target = Some(self.display.draw());
        true
    }

    fn events(&mut self) {

        for ev in self.display.poll_events() {
            match ev {
                glutin::Event::Closed => {
                    self.is_running = false;
                },
                glutin::Event::Focused(false) => {
                    for i in 0..256 {
                        self.key_state[i] = false;
                    }
                },
                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, code, _) => {
                    self.key_state[code as usize] = true;

                    if code == KeyCode::C as u8 && self.key_state[KeyCode::LCtrl as usize] {
                        self.is_running = false;
                    }
                },
                glutin::Event::KeyboardInput(glutin::ElementState::Released, code, _) => {
                    self.key_state[code as usize] = false;
                },
                _ => {}
            }
        }

        self.set_time(clock_ticks::precise_time_ms() as f64 / 1000.0);

    }

    fn running(&mut self) -> bool {
        self.is_running
    }

    fn draw(&mut self) {
        let target = self.target.take().unwrap();
        target.finish().unwrap();
        self.display.swap_buffers().unwrap(); // TODO make sure vsync doesn't screw with us here
        self.key_state_old = self.key_state;
    }

}

