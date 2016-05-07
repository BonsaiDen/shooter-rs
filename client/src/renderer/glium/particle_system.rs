// External Dependencies ------------------------------------------------------
use glium;
use glium::Surface;


// Internal Dependencies ------------------------------------------------------
use renderer::{ParticleSystem, Particle};


// Glium based ParticleSystem -------------------------------------------------
pub struct GliumParticleSystem {

    system: ParticleSystem,

    vertex_particles: Vec<Vertex>,
    vertices: glium::VertexBuffer<Vertex>,
    indices: glium::index::NoIndices,

    params: glium::DrawParameters<'static>,
    program: glium::Program

}

impl GliumParticleSystem {

    pub fn new(display: &glium::backend::glutin_backend::GlutinFacade, max_particles: usize) -> GliumParticleSystem {

        let mut vertex_particles = Vec::with_capacity(max_particles);
        for _ in 0..max_particles {
            vertex_particles.push(Vertex {
                position: [-1.0, 1.0],
                scale: 0.0,
                color: [1.0, 0.0, 0.0, 1.0]
            })
        }

        let vertices = glium::VertexBuffer::new(display, &vertex_particles).unwrap();

        GliumParticleSystem {

            system: ParticleSystem::new(max_particles),

            vertex_particles: vertex_particles,
            vertices: vertices,
            indices: glium::index::NoIndices(glium::index::PrimitiveType::Points),

            params: glium::DrawParameters {
                polygon_mode: glium::draw_parameters::PolygonMode::Point,
                blend: glium::Blend {
                    color: glium::BlendingFunction::Addition {
                        source: glium::LinearBlendingFactor::SourceAlpha,
                        destination: glium::LinearBlendingFactor::OneMinusSourceAlpha
                    },
                    .. Default::default()
                },
                .. Default::default()
            },

            program: glium::Program::new(
                display,
                glium::program::ProgramCreationInput::SourceCode {
                    tessellation_control_shader: None,
                    tessellation_evaluation_shader: None,
                    geometry_shader: None,
                    outputs_srgb: false,
                    uses_point_size: true,
                    vertex_shader: r#"
                        #version 140

                        in vec2 position;
                        in float scale;
                        in vec4 color;
                        out vec4 colorV;

                        uniform mat4 perspective;

                        void main() {
                            colorV = color;
                            gl_PointSize = scale;
                            gl_Position = perspective * vec4(position, 0.0, 1.0);
                        }
                    "#,
                    fragment_shader: r#"
                        #version 140

                        in vec4 colorV;
                        out vec4 outColor;

                        void main() {
                            outColor = colorV;
                        }
                    "#,
                    transform_feedback_varyings: None
                }
            ).unwrap()

        }

    }

    pub fn get(&mut self) -> Option<&mut Particle> {
        self.system.get()
    }

    pub fn draw(
        &mut self, target: &mut glium::Frame, pm: &[[f32; 4]; 4], dt: f32
    ) {

        let particles = &mut self.vertex_particles;
        self.system.draw(dt, |i, ref particle, alpha| {
            let mut vertex = particles.get_mut(i).unwrap();
            vertex.position[0] = particle.x;
            vertex.position[1] = particle.y;
            vertex.scale = particle.s;
            vertex.color[0] = particle.color.r as f32 / 255.0;
            vertex.color[1] = particle.color.g as f32 / 255.0;
            vertex.color[2] = particle.color.b as f32 / 255.0;
            vertex.color[3] = alpha / 255.0;
        });

        self.vertices.write(particles);

        let uniforms = uniform! {
            perspective: *pm
        };

        target.draw(
            &self.vertices, &self.indices, &self.program, &uniforms,
            &self.params

        ).unwrap();

    }

}

// Particle Vertex ------------------------------------------------------------
#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub scale: f32,
    pub color: [f32; 4]
}

implement_vertex!(Vertex, position, scale, color);

