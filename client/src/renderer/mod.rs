#[cfg(feature="allegro_renderer")]
mod allegro;

#[cfg(feature="allegro_renderer")]
pub use self::allegro::AllegroRenderer as Renderer;

