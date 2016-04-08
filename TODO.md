# Renderer Abstraction

- Abstract Base Renderer (Timing etc.) into BaseRenderer
- Split Particle Rendering into Logic Component and Drawing Component

- ParticleRenderer
    self.particle_system

- Abstract Entity Rendering so that non-immediate rendering is possible
    - Have shape methods in renderer? 
    - Or have different implementations of the entity?


# IdPool

- Better solution to prevent immediate ID re-use which could cause problems 
  with out of order receival of messages

# Both

- Remove Renderer::run() trait method?
- Timer use milliseconds or time::Duration?


# Client

- Add colored border to screen to indicate player color
- Clean up renderer encapsulation a bit more


# Server

- Event visibility

