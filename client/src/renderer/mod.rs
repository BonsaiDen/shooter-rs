// Allegro Rendering Implementation -------------------------------------------
#[cfg(feature="allegro_renderer")]
mod allegro;

#[cfg(feature="allegro_renderer")]
pub use self::allegro::AllegroRenderer as Renderer;

#[cfg(feature="allegro_renderer")]
pub use self::allegro::KeyCode as KeyCode;

// GLium Rendering Implementation ---------------------------------------------
#[cfg(feature="glium_renderer")]
mod glium;

#[cfg(feature="glium_renderer")]
pub use self::glium::GliumRenderer as Renderer;

#[cfg(feature="glium_renderer")]
pub use self::glium::KeyCode as KeyCode;

// Generic Components ---------------------------------------------------------
mod particle_system;
pub use self::particle_system::ParticleSystem as ParticleSystem;
pub use self::particle_system::Particle as Particle;

