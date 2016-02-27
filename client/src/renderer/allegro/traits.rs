// External Dependencies ------------------------------------------------------
use std::any::Any;
use allegro::{
    Core,
    Display,
    DisplayOption,
    DisplayOptionImportance,
    EventQueue,
    OPENGL
};


// Internal Dependencies ------------------------------------------------------
use lithium;
use super::Renderer;


// Allegro Renderer Trait Implementation --------------------------------------
impl lithium::Renderer for Renderer {

    // Statics ----------------------------------------------------------------
    fn run<R: lithium::Runnable<E, L>, E: lithium::event::Event, L: lithium::Level>(
        mut client: lithium::Client<E, L>,
        mut runnable: R

    ) where Self: Sized {

        // Init Allegro
        let mut core = Core::init().unwrap();
        let q = EventQueue::new(&core).unwrap();

        // Keyboard
        core.install_keyboard().unwrap();
        q.register_event_source(core.get_keyboard_event_source());

        // Create Display
        core.set_new_display_flags(OPENGL);
        core.set_new_display_option(
            DisplayOption::SampleBuffers,
            2,
            DisplayOptionImportance::Suggest
        );

        core.set_new_display_option(
            DisplayOption::Samples,
            16,
            DisplayOptionImportance::Suggest
        );

        let disp = Display::new(
            &core, 256, 256

        ).ok().expect("Failed to create OPENGL context.");

        q.register_event_source(disp.get_event_source());

        // Create renderer
        let mut renderer = Renderer::new(core, disp, q);

        // Init callback
        client.init(&mut runnable, &mut renderer);

        // Mainloop
        let mut last_tick_time = 0.0;
        let mut last_frame_time = 0.0;
        let mut frames_per_tick = 0;

        while renderer.running() {

            if renderer.should_draw() {

                let frame_time = renderer.time();
                let tick_rate = renderer.tick_rate();

                if frames_per_tick == 0 {
                    if client.tick(&mut runnable, &mut renderer) {
                        frames_per_tick = renderer.fps() / tick_rate;
                        last_tick_time = frame_time;
                    }
                }

                renderer.set_delta_time((frame_time - last_frame_time) as f32);
                renderer.set_delta_u(
                    1.0 / (1.0 / tick_rate as f32) * (frame_time - last_tick_time) as f32
                );

                client.draw(&mut runnable, &mut renderer);
                renderer.draw();

                last_frame_time = frame_time;
                frames_per_tick -= 1;

            }

            renderer.events();

        }

        client.destroy(&mut runnable, &mut renderer);

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

