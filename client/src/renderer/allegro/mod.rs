// External Dependencies ------------------------------------------------------
use allegro::{
    Core,
    Color as AllegroColor,
    Display,
    DisplayClose,
    EventQueue,
    KeyCode,
    KeyUp,
    KeyDown,
    Timer,
    TimerTick
};
use rand::{SeedableRng, XorShiftRng};
use allegro_font::{FontDrawing, FontAddon, Font, FontAlign};
use allegro_primitives::PrimitivesAddon;


// Internal Dependencies ------------------------------------------------------
mod traits;
use shared::Color;
use shared::Lithium::Renderer;
use renderer::particle::{Particle, ParticleSystem};


// Allegro Based Renderer -----------------------------------------------------
pub struct AllegroRenderer {

    // Allegro Related
    core: Core,
    display: Display,
    queue: EventQueue,
    prim: PrimitivesAddon,
    timer: Timer,
    font: Font,

    // Timing
    frame_rate: u32,
    tick_rate: u32,
    time: f64,
    dt: f32,
    u: f32,

    // Drawing
    particle_system: ParticleSystem,
    interpolation_ticks: usize,

    // Input
    key_state: [bool; 256],
    key_state_old: [bool; 256],

    // Internal State
    rng: XorShiftRng,
    is_running: bool,
    redraw: bool

}

impl AllegroRenderer {

    pub fn new(
        core: Core, display: Display, queue: EventQueue

    ) -> AllegroRenderer {

        let prim = PrimitivesAddon::init(&core).unwrap();
        let font_addon = FontAddon::init(&core).unwrap();
        let font = Font::new_builtin(&font_addon).unwrap();
        let timer = Timer::new(&core, 1.0 / 60.0).unwrap();

        queue.register_event_source(timer.get_event_source());
        timer.start();

        AllegroRenderer {
            core: core,
            display: display,
            queue: queue,
            prim: prim,
            timer: timer,
            font: font,
            particle_system: ParticleSystem::new(1000),
            is_running: true,
            redraw: false,
            frame_rate: 60,
            tick_rate: 60,
            time: 0.0,
            dt: 0.0,
            u: 0.0,
            key_state: [false; 256],
            key_state_old: [false; 256],
            rng: XorShiftRng::new_unseeded(),
            interpolation_ticks: 0
        }

    }


    // Window Handling --------------------------------------------------------
    pub fn set_title(&mut self, title: &str) {
        self.display.set_window_title(title);
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.display.resize(width, height).ok();
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
        self.core.clear_to_color(AllegroRenderer::get_color(color));
    }

    pub fn triangle(
        &mut self, color: &Color,
        ax: f32, ay: f32,
        bx: f32, by: f32,
        cx: f32, cy: f32,
        line_width: f32
    ) {
        self.prim.draw_triangle(
            ax, ay, bx, by, cx, cy, AllegroRenderer::get_color(color), line_width
        );
    }

    pub fn text(&mut self, color: &Color, x: f32, y: f32, text: &str) {
        self.core.draw_text(
            &self.font, AllegroRenderer::get_color(color), x, y, FontAlign::Left, text
        );
    }

    pub fn particle(&mut self) -> Option<&mut Particle> {
        self.particle_system.get()
    }

    pub fn draw_particles(&mut self) {
        let prim = &self.prim;
        self.particle_system.draw(self.dt, |ref color, s, x, y| {
            prim.draw_filled_rectangle(
                x - s + 0.5, y - s + 0.5, x + s + 0.5, y + s + 0.5,
                AllegroRenderer::get_color(color)
            );
        });
    }


    // RNG --------------------------------------------------------------------
    pub fn reseed_rng(&mut self, seed: [u32; 4]) {
        self.rng.reseed(seed);
    }

    pub fn rng(&mut self) -> &mut XorShiftRng {
        &mut self.rng
    }


    // Color Conversion -------------------------------------------------------
    pub fn get_color(color: &Color) -> AllegroColor {
        let a = color.a as f32 / 255.0;
        AllegroColor::from_rgb(
            (color.r as f32 * a) as u8,
            (color.g as f32 * a) as u8,
            (color.b as f32 * a) as u8
        )
    }

}


// Internal Methods required for trait implementation -------------------------
impl AllegroRenderer {

    fn should_draw(&mut self) -> bool {
        let redraw = self.redraw;
        self.redraw = false;
        redraw
    }

    fn events(&mut self) {

        match self.queue.wait_for_event() {

            DisplayClose{ ..} => {
                self.is_running = false;
            },

            KeyDown{keycode: k, ..} if (k as u32) < 255 => {

                self.key_state[k as usize] = true;

                //println!("key down {:?}", k as usize);

                // Exit via Ctrl-C
                if k == KeyCode::C && self.key_state[KeyCode::LCtrl as usize] {
                    self.is_running = false;
                }

            },

            KeyUp{keycode: k, ..} if (k as u32) < 255 => {
                //println!("key up {:?}", k as usize);
                self.key_state[k as usize] = false;
            },

            TimerTick{timestamp: t, ..} => {
                self.set_time(t);
                self.redraw = true;
            },

            _ => ()

        }
    }

    fn running(&mut self) -> bool {
        self.is_running
    }

    fn draw(&mut self) {
        self.core.flip_display();
        self.key_state_old = self.key_state;
    }

}

