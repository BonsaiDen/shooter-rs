// Internal Dependencies ------------------------------------------------------
use client::ClientProxy;
use entity::{Entity, ControlState};
use event::Event;
use level::Level;
use renderer::Renderer;


// Runnable Abstraction -------------------------------------------------------
pub trait Runnable<E: Event, L: Level> {

    fn init(&mut self, &mut Renderer, ClientProxy<L>);
    fn connect(&mut self, &mut Renderer, ClientProxy<L>);
    fn disconnect(&mut self, &mut Renderer, ClientProxy<L>);

    fn level(&mut self, &mut Renderer, &[u8]) -> L;
    fn config(&mut self, &mut Renderer, ClientProxy<L>);

    fn event(&mut self, &mut Renderer, ClientProxy<L>, E);
    fn tick_before(&mut self, &mut Renderer, ClientProxy<L>, u8, f32);
    fn tick_entity_before(&mut self, &mut Renderer, &mut Entity, &L, u8, f32);
    fn tick_entity_after(&mut self, &mut Renderer, &mut Entity, &L, u8, f32) -> ControlState;
    fn tick_after(&mut self, &mut Renderer, ClientProxy<L>, u8, f32);

    fn draw(&mut self, &mut Renderer, ClientProxy<L>);

    fn destroy(&mut self, ClientProxy<L>) {}

}

