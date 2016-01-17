// Internal Dependencies ------------------------------------------------------
use renderer::Renderer;


// Runnable Abstraction -------------------------------------------------------
pub trait Runnable {
    fn init(&mut self, &mut Renderer);
    fn tick(&mut self, &mut Renderer) -> bool;
    fn draw(&mut self, &mut Renderer);
    fn destroy(&mut self);
}

