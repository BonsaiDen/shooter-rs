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

use game::GameEvents;
use shared::color::Color;
use shared::renderer::Renderer;
use shared::particle::{Particle, ParticleSystem};

pub struct AllegroRenderContainer;

impl AllegroRenderContainer {

    pub fn new() -> AllegroRenderContainer {
        AllegroRenderContainer
    }

    pub fn run<G: GameEvents>(&self, mut game: G)  {

        // Init Allegro
        let mut core = Core::init().unwrap();
        let q = EventQueue::new(&core).unwrap();

        // Keyboard
        core.install_keyboard().unwrap();
        q.register_event_source(core.get_keyboard_event_source());

        // Create Display
        core.set_new_display_flags(allegro::OPENGL);
        core.set_new_display_option(DisplayOption::SampleBuffers, 1, DisplayOptionImportance::Require);
        core.set_new_display_option(DisplayOption::Samples, 16, DisplayOptionImportance::Require);

        let disp = Display::new(&core, 256, 256).ok().expect("Failed to create OPENGL context.");
        q.register_event_source(disp.get_event_source());

        // Create renderer
        let mut renderer = AllegroRenderer::new(core, disp, q);

        // Init callback
        game.init(&mut renderer);

        // Mainloop
        let mut last_tick_time = 0.0;
        let mut last_frame_time = 0.0;
        let mut frames_per_tick = 0;

        while renderer.running() {

            if renderer.do_draw() {

                let frame_time = renderer.get_time();
                let tick_rate = renderer.get_tick_rate();

                if frames_per_tick == 0 {
                    if game.tick(&mut renderer) {
                        frames_per_tick = renderer.get_fps() / tick_rate;
                        last_tick_time = frame_time;
                    }
                }

                renderer.set_delta_time((frame_time - last_frame_time) as f32);
                renderer.set_delta_u(
                    1.0 / (1.0 / tick_rate as f32) * (frame_time - last_tick_time) as f32
                );

                game.draw(&mut renderer);

                last_frame_time = frame_time;
                frames_per_tick -= 1;

            }

            renderer.events();

        }

        game.destroy();

    }

}

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
    key_state: [bool; 255]
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
            key_state: [false; 255]
        }

    }
}

impl Renderer for AllegroRenderer {

    fn set_fps(&mut self, frame_rate: u32) {
        self.frame_rate = frame_rate;
        self.timer.set_speed(1.0 / frame_rate as f64);
    }

    fn get_fps(&mut self) -> u32 {
        self.frame_rate
    }

    fn set_time(&mut self, time: f64) {
        self.time = time;
    }

    fn get_time(&mut self) -> f64 {
        self.time
    }

    fn set_tick_rate(&mut self, tick_rate: u32) {
        self.tick_rate = tick_rate;
    }

    fn get_tick_rate(&mut self) -> u32 {
        self.tick_rate
    }

    fn set_delta_time(&mut self, dt: f32) {
        self.dt = dt;
    }

    fn set_delta_u(&mut self, u: f32) {
        self.u = u;
    }

    fn get_delta_time(&mut self) -> f32{
        self.dt
    }

    fn get_delta_u(&mut self) -> f32 {
        self.u
    }

    fn set_title(&mut self, title: &str) {
        self.display.set_window_title(title);
    }

    fn do_draw(&mut self) -> bool {
        let redraw = self.redraw;
        self.redraw = false;
        redraw
    }

    fn running(&mut self) -> bool {
        self.is_running
    }

    fn key_down(&mut self, key_code: u8) -> bool {
        self.key_state[key_code as usize]
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

    fn resize(&mut self, width: i32, height: i32) {
        self.display.resize(width, height).ok();
    }

    fn clear(&mut self, color: &Color) {
        self.core.clear_to_color(get_color(color));
    }

    fn draw(&mut self, dt: f32, _: f32) {
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
        self.prim.draw_triangle(
            ax, ay, bx, by, cx, cy, get_color(color), line_width
        );
    }

    fn text(&mut self, color: &Color, x: f32, y: f32, text: &str) {
        self.core.draw_text(
            &self.font, get_color(color), x, y, FontAlign::Left, text
        );
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

