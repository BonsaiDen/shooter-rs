// Allegro Rendering Implementation -------------------------------------------
#[cfg(feature="allegro_renderer")]
mod allegro;

#[cfg(feature="allegro_renderer")]
pub use self::allegro::AllegroRenderer as Renderer;

// GLium Rendering Implementation ---------------------------------------------
#[cfg(feature="glium_renderer")]
mod glium;

#[cfg(feature="glium_renderer")]
pub use self::glium::GliumRenderer as Renderer;

// Generic Components ---------------------------------------------------------
mod particle_system;
pub use self::particle_system::ParticleSystem as ParticleSystem;
pub use self::particle_system::Particle as Particle;

