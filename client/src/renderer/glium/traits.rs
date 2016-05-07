// External Dependencies ------------------------------------------------------
use glium::{glutin, DisplayBuild, Surface};


// Internal Dependencies ------------------------------------------------------
use shared::Lithium::{
    Client, ClientHandler, EntityState, EntityRegistry, Event, BaseLevel, Renderer
};
use super::GliumRenderer;


// Glium Renderer Trait Implementation ----------------------------------------
impl Renderer for GliumRenderer {

    // Statics ----------------------------------------------------------------
    fn run<
        H: ClientHandler<Self, G, L, E, S>,
        E: Event,
        S: EntityState,
        L: BaseLevel<S>,
        G: EntityRegistry<S, L, Self>

    >(mut client: Client<H, Self, G, L, E, S>) where Self: Sized {

        let (width, height) = (256, 256);
        let display = glutin::WindowBuilder::new()
            .with_multisampling(4)
            .with_dimensions(width, height)
            //.with_max_dimensions(width, height)
            //.with_min_dimensions(width, height)
            //.with_title(format!("Test!"))
            .build_glium().unwrap();

        // Create renderer
        let mut renderer = GliumRenderer::new(display, width, height);

        // Init callback
        client.init(&mut renderer);

        // Mainloop
        let mut last_tick_time = 0.0;
        let mut last_frame_time = 0.0;
        let mut frames_per_tick = 0;

        while renderer.running() {

            if renderer.should_draw() {

                let frame_time = renderer.time();
                let tick_rate = renderer.tick_rate();

                if frames_per_tick == 0 {
                    if client.tick(&mut renderer) {
                        frames_per_tick = renderer.fps() / tick_rate;
                        last_tick_time = frame_time;
                    }
                }

                renderer.set_delta_time((frame_time - last_frame_time) as f32);
                renderer.set_delta_u(
                    1.0 / (1.0 / tick_rate as f32) * (frame_time - last_tick_time) as f32
                );

                client.draw(&mut renderer);
                renderer.draw();

                last_frame_time = frame_time;

                // TODO handle this more nicely?
                if frames_per_tick > 0 {
                    frames_per_tick -= 1;
                }

            }

            renderer.events();

        }

        client.destroy(&mut renderer);

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
        // TODO set timer / frame rate?
        //self.timer.set_speed(1.0 / frame_rate as f64);
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

