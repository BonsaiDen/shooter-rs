// Internal Dependencies ------------------------------------------------------
use client::ClientProxy;
use entity::{Entity, ControlState};
use event::Event;
use level::Level;
use renderer::Renderer;


// Runnable Abstraction -------------------------------------------------------
pub trait Runnable<E: Event, L: Level> {

    fn init(&mut self, ClientProxy<L>);
    fn connect(&mut self, ClientProxy<L>);
    fn disconnect(&mut self, ClientProxy<L>);

    fn level(&mut self, ClientProxy<L>, &[u8]) -> L;
    fn config(&mut self, ClientProxy<L>);

    fn event(&mut self, ClientProxy<L>, E);
    fn tick_before(&mut self, ClientProxy<L>, u8, f32);
    fn tick_entity_before(&mut self, &mut Renderer, &mut Entity, &L, u8, f32);
    fn tick_entity_after(&mut self, &mut Renderer, &mut Entity, &L, u8, f32) -> ControlState;
    fn tick_after(&mut self, ClientProxy<L>, u8, f32);

    fn draw(&mut self, ClientProxy<L>);

    fn destroy(&mut self, ClientProxy<L>) {}

}

