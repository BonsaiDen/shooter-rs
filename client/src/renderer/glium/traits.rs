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

