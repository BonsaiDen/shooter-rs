// Internal -------------------------------------------------------------------
use event::Event;
use level::BaseLevel;
use entity::{EntityState, EntityRegistry};
use client::{Client, Handler};


// Renderer Abstraction -------------------------------------------------------
pub trait Renderer {

    // Statics ----------------------------------------------------------------
    fn run<
        H: Handler<E, S, L, Self, G>,
        E: Event,
        S: EntityState,
        L: BaseLevel<S>,
        G: EntityRegistry<S, L, Self>

    >(_: Client<E, S, L, Self, H, G>) where Self: Sized {}

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

}


// Default Noop Renderer Implementation ---------------------------------------
pub struct DefaultRenderer;
impl Renderer for DefaultRenderer {

    // Time Related -----------------------------------------------------------
    fn time(&self) -> f64 {
        0.0
    }

    fn set_time(&mut self, _: f64) {
    }

    fn delta_time(&self) -> f32 {
        0.0
    }

    fn set_delta_time(&mut self, _: f32) {
    }

    fn delta_u(&self) -> f32 {
        0.0
    }

    fn set_delta_u(&mut self, _: f32) {
    }


    // Frame / Tick Rate ------------------------------------------------------
    fn fps(&self) -> u32 {
        0
    }

    fn set_fps(&mut self, _: u32) {
    }

    fn tick_rate(&self) -> u32 {
        0
    }

    fn set_tick_rate(&mut self, _: u32) {
    }


    // Interpolation ----------------------------------------------------------
    fn interpolation_ticks(&self) -> usize {
        0
    }

    fn set_interpolation_ticks(&mut self, _: usize) {
    }

}

