// External Dependencies ------------------------------------------------------
use rand::XorShiftRng;
use std::any::Any;


// Internal -------------------------------------------------------------------
use runnable::Runnable;


// Renderer Abstraction -------------------------------------------------------
pub trait Renderer {

    fn run<R: Runnable>(runnable: R) where Self: Sized;

    fn as_any(&mut self) -> &mut Any;

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


    // RNG --------------------------------------------------------------------
    fn reseed_rng(&mut self, seed: [u32; 4]);

    fn rng(&mut self) -> &mut XorShiftRng;

}
