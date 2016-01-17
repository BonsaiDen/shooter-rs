// External Dependencies ------------------------------------------------------
use rand::{SeedableRng, XorShiftRng};
use std::any::Any;

// Allegro Dependencies -------------------------------------------------------
use allegro;
use allegro::{
    Core,
    Color as AllegroColor,
    Display,
    DisplayOption,
    DisplayOptionImportance,
    EventQueue,
    KeyCode,
    Timer
};
use allegro_font::{FontDrawing, FontAddon, Font, FontAlign};
use allegro_primitives::PrimitivesAddon;


// Internal Dependencies ------------------------------------------------------
use shared::color::Color;
use shared::particle::{Particle, ParticleSystem};
use lithium::{Renderer, Runnable};


// Allegro Based Renderer -----------------------------------------------------
pub struct AllegroRenderer {
    core: Core,
    display: Display,
    queue: EventQueue,
    prim: PrimitivesAddon,
    timer: Timer,
    font: Font,
    particle_system: ParticleSystem,
    is_running: bool,
    redraw: bool,
    frame_rate: u32,
    tick_rate: u32,
    time: f64,
    dt: f32,
    u: f32,
    key_state: [bool; 255],
    rng: XorShiftRng,
    interpolation_ticks: usize
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
            key_state: [false; 255],
            rng: XorShiftRng::new_unseeded(),
            interpolation_ticks: 0
        }

    }

    pub fn get<'a>(renderer: &'a mut Renderer) -> &'a mut AllegroRenderer {
        match renderer.as_any().downcast_mut::<AllegroRenderer>() {
            Some(r) => r,
            None => unreachable!()
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


    // Drawing Methods --------------------------------------------------------
    pub fn clear(&mut self, color: &Color) {
        self.core.clear_to_color(get_color(color));
    }

    pub fn triangle(
        &mut self, color: &Color,
        ax: f32, ay: f32,
        bx: f32, by: f32,
        cx: f32, cy: f32,
        line_width: f32
    ) {
        self.prim.draw_triangle(
            ax, ay, bx, by, cx, cy, get_color(color), line_width
        );
    }

    pub fn text(&mut self, color: &Color, x: f32, y: f32, text: &str) {
        self.core.draw_text(
            &self.font, get_color(color), x, y, FontAlign::Left, text
        );
    }

    pub fn particle(&mut self) -> Option<&mut Particle> {
        self.particle_system.get()
    }


    // RNG --------------------------------------------------------------------
    pub fn reseed_rng(&mut self, seed: [u32; 4]) {
        self.rng.reseed(seed);
    }

    pub fn rng(&mut self) -> &mut XorShiftRng {
        &mut self.rng
    }


    // Private ----------------------------------------------------------------
    fn should_draw(&mut self) -> bool {
        let redraw = self.redraw;
        self.redraw = false;
        redraw
    }

    fn events(&mut self) {
        match self.queue.wait_for_event() {

            allegro::DisplayClose{ ..} => {
                self.is_running = false;
            },

            allegro::KeyDown{keycode: k, ..} if (k as u32) < 255 => {

                self.key_state[k as usize] = true;

                // Exit via Ctrl-C
                if k == KeyCode::C && self.key_state[KeyCode::LCtrl as usize] {
                    self.is_running = false;
                }

            },

            allegro::KeyUp{keycode: k, ..} if (k as u32) < 255 => {
                self.key_state[k as usize] = false;
            },

            allegro::TimerTick{timestamp: t, ..} => {
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
        let prim = &self.prim;
        self.particle_system.draw(self.dt, |ref color, s, x, y| {
            prim.draw_filled_rectangle(
                x - s + 0.5, y - s + 0.5, x + s + 0.5, y + s + 0.5,
                get_color(color)
            );
        });
        self.core.flip_display();
    }

}

impl Renderer for AllegroRenderer {

    // Statics ----------------------------------------------------------------
    fn run<R: Runnable>(mut runnable: R) where Self: Sized {

        // Init Allegro
        let mut core = Core::init().unwrap();
        let q = EventQueue::new(&core).unwrap();

        // Keyboard
        core.install_keyboard().unwrap();
        q.register_event_source(core.get_keyboard_event_source());

        // Create Display
        core.set_new_display_flags(allegro::OPENGL);
        core.set_new_display_option(DisplayOption::SampleBuffers, 2, DisplayOptionImportance::Suggest);
        core.set_new_display_option(DisplayOption::Samples, 16, DisplayOptionImportance::Suggest);

        let disp = Display::new(&core, 256, 256).ok().expect("Failed to create OPENGL context.");
        q.register_event_source(disp.get_event_source());

        // Create renderer
        let mut renderer = AllegroRenderer::new(core, disp, q);

        // Init callback
        runnable.init(&mut renderer);

        // Mainloop
        let mut last_tick_time = 0.0;
        let mut last_frame_time = 0.0;
        let mut frames_per_tick = 0;

        while renderer.running() {

            if renderer.should_draw() {

                let frame_time = renderer.time();
                let tick_rate = renderer.tick_rate();

                if frames_per_tick == 0 {
                    if runnable.tick(&mut renderer) {
                        frames_per_tick = renderer.fps() / tick_rate;
                        last_tick_time = frame_time;
                    }
                }

                renderer.set_delta_time((frame_time - last_frame_time) as f32);
                renderer.set_delta_u(
                    1.0 / (1.0 / tick_rate as f32) * (frame_time - last_tick_time) as f32
                );

                runnable.draw(&mut renderer);
                renderer.draw();

                last_frame_time = frame_time;
                frames_per_tick -= 1;

            }

            renderer.events();

        }

        runnable.destroy();

    }


    // Downcast ---------------------------------------------------------------
    fn as_any(&mut self) -> &mut Any {
        self
    }


    // Time Related -----------------------------------------------------------
    fn time(&self) -> f64 {
        self.time
    }

    fn set_time(&mut self, time: f64) {
        self.time = time;
    }

    fn delta_time(&self) -> f32{
        self.dt
    }

    fn set_delta_time(&mut self, dt: f32) {
        self.dt = dt;
    }

    fn delta_u(&self) -> f32 {
        self.u
    }

    fn set_delta_u(&mut self, u: f32) {
        self.u = u;
    }


    // Frame / Tick Rate ------------------------------------------------------
    fn fps(&self) -> u32 {
        self.frame_rate
    }

    fn set_fps(&mut self, frame_rate: u32) {
        self.frame_rate = frame_rate;
        self.timer.set_speed(1.0 / frame_rate as f64);
    }

    fn tick_rate(&self) -> u32 {
        self.tick_rate
    }

    fn set_tick_rate(&mut self, tick_rate: u32) {
        self.tick_rate = tick_rate;
    }


    // Interpolation ----------------------------------------------------------
    fn interpolation_ticks(&self) -> usize {
        self.interpolation_ticks
    }

    fn set_interpolation_ticks(&mut self, ticks: usize) {
        self.interpolation_ticks = ticks;
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

