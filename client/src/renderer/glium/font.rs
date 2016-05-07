// STD Dependencies -----------------------------------------------------------
use std::cmp;
use std::fs::File;


// External Dependencies ------------------------------------------------------
use image;
use glium;
use glium::Surface;
use bmfont::{BMFont, OrdinateOrientation};


// Statics --------------------------------------------------------------------
const MAX_CHARS: usize = 256;


// BMFont Abstraction ---------------------------------------------------------
pub struct Font {

    last_length: usize,

    bm_font: BMFont,

    scale: f32,
    tex_size: f32,
    texture: glium::texture::unsigned_texture2d::UnsignedTexture2d,

    vertex_chars: Vec<Vertex>,
    vertices: glium::VertexBuffer<Vertex>,
    indices: glium::index::NoIndices,

    params: glium::DrawParameters<'static>,
    program: glium::Program

}

impl Font {

    pub fn new(display: &glium::backend::glutin_backend::GlutinFacade, font_file: &str, image_file: &str) -> Font {

        let img = image::load(File::open(image_file).unwrap(), image::PNG).unwrap().to_rgba();
        let image_dimensions = img.dimensions();
        let img = glium::texture::RawImage2d::from_raw_rgba_reversed(img.into_raw(), image_dimensions);
        let bm_font = BMFont::new(File::open(font_file).unwrap(), OrdinateOrientation::TopToBottom).unwrap();
        let scale = (bm_font.line_height() + 1) as f32;

        let vertex_chars = char_vertices(MAX_CHARS);
        let vertices = glium::VertexBuffer::new(display, &vertex_chars).unwrap();

        Font {

            bm_font: bm_font,
            last_length: 0,

            scale: scale,
            tex_size: image_dimensions.0 as f32,
            texture: glium::texture::unsigned_texture2d::UnsignedTexture2d::new(
                display,
                img

            ).unwrap(),

            vertex_chars: vertex_chars,
            vertices: vertices,
            indices: glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),

            params: glium::DrawParameters {
                dithering: false,
                smooth: Some(glium::Smooth::Fastest),
                blend: glium::Blend {
                    color: glium::BlendingFunction::Addition {
                        source: glium::LinearBlendingFactor::SourceAlpha,
                        destination: glium::LinearBlendingFactor::OneMinusSourceAlpha
                    },
                    .. Default::default()
                },
                .. Default::default()
            },
            program: glium::Program::from_source(
                display,
                r#"
                    #version 140

                    uniform mat4 perspective;
                    uniform float texSize;
                    uniform vec2 offset;
                    uniform float scale;

                    attribute vec4 position;
                    attribute vec4 char_coords;

                    out vec2 tc;

                    void main() {

                        mat4 matrix;
                        matrix[0][0] = scale;
                        matrix[1][1] = scale;
                        matrix[2][2] = scale;
                        matrix[3][0] = position[2];
                        matrix[3][1] = position[3];
                        matrix[3][3] = 1.0;

                        // Transform from absolute texture coordinates to normalized texture coordinates
                        // This works because the rectangle spans [0,1] x [0,1]
                        // Depending on where the origin lies in your texture (i.e. topleft or bottom left corner),
                        // you need to replace "1. - position.y" with just "position.y"
                        tc = (char_coords.xy + char_coords.zw * vec2(position.x, 1.0 - position.y)) / texSize;

                        // Map the vertices of the unit square to a rectangle with
                        // correct aspect ratio and positioned at the correct offset
                        float x = (char_coords[2] * position.x + offset.x) / char_coords[3];
                        float y = position.y + offset.y / char_coords[3];

                        // Apply the model, view and projection transformations
                        gl_Position = perspective * matrix * vec4(x, y, 0.0, 1.0);

                    }
                "#,
                r#"
                    #version 140

                    uniform vec4 color;
                    uniform usampler2D tex;

                    in vec2 tc;
                    out vec4 outColor;

                    void main() {
                        uvec4 vec_tex;
                        vec_tex = texture(tex, tc);
                        outColor = color * vec4(1.0, 1.0, 1.0, float(vec_tex.x) / 255.0);
                    }
                "#,
                None

            ).unwrap()
        }
    }

    //pub fn text(&self) -> Text {

    //}

    pub fn draw(
        &mut self, target: &mut glium::Frame,
        pm: &[[f32; 4]; 4], text: &str, x: f32, y: f32, color: [f32; 4]
    ) {

        // TODO support alignment
        // TODO allow partial write?

        // TODO add a text type which contains the pre-parsed text,
        // this can speed up commonly used text segments
        // dynamic parts should be placed on their own?
        let positions = self.bm_font.parse(
            &text[0..cmp::min(text.len(), MAX_CHARS)]

        ).unwrap();

        // Clear previously used vertices
        if self.last_length > positions.len() {
            for i in positions.len()..self.last_length {
                let mut vertex = &mut self.vertex_chars[i * 6..i * 6 + 6];
                let py = -100.0;
                vertex[0].position[3] = py;
                vertex[1].position[3] = py;
                vertex[2].position[3] = py;
                vertex[3].position[3] = py;
                vertex[4].position[3] = py;
                vertex[5].position[3] = py;
            }
        }

        self.last_length = positions.len();

        // Render new text
        for (i, p) in positions.iter().enumerate() {

            let px = p.screen_rect.x as f32 + x + 0.5;
            let py = p.screen_rect.y as f32 + y + 0.5;

            let char_coords: [f32; 4] = [
                p.page_rect.x as f32,
                self.tex_size - p.page_rect.y as f32 - p.page_rect.height as f32,
                p.page_rect.width as f32,
                p.page_rect.height as f32
            ];

            // a--b     d
            // |        |
            // c     c--e
            let mut vertex = &mut self.vertex_chars[i * 6..i * 6 + 6];

            vertex[0].position[2] = px;
            vertex[1].position[2] = px;
            vertex[2].position[2] = px;
            vertex[3].position[2] = px;
            vertex[4].position[2] = px;
            vertex[5].position[2] = px;

            vertex[0].position[3] = py;
            vertex[1].position[3] = py;
            vertex[2].position[3] = py;
            vertex[3].position[3] = py;
            vertex[4].position[3] = py;
            vertex[5].position[3] = py;

            vertex[0].char_coords = char_coords;
            vertex[1].char_coords = char_coords;
            vertex[2].char_coords = char_coords;
            vertex[3].char_coords = char_coords;
            vertex[4].char_coords = char_coords;
            vertex[5].char_coords = char_coords;

        }

        self.vertices.write(&self.vertex_chars);

        let uniforms = uniform! {
            perspective: *pm,
            color: color,
            scale: self.scale,
            texSize: self.tex_size,
            offset: [0.0, 0.0f32],
            tex: &self.texture
        };

        target.draw(
            &self.vertices, &self.indices, &self.program, &uniforms,
            &self.params

        ).unwrap();

    }

}

fn char_vertices(max_chars: usize) -> Vec<Vertex> {

    let mut buffer = Vec::with_capacity(max_chars * 6);
    for _ in 0..max_chars {
        buffer.push(Vertex { position: [0.0, 0.0, 0.0, 0.0], char_coords: [0.0, 0.0, 0.0, 0.0] });
        buffer.push(Vertex { position: [1.0, 0.0, 0.0, 0.0], char_coords: [0.0, 0.0, 0.0, 0.0] });
        buffer.push(Vertex { position: [0.0, 1.0, 0.0, 0.0], char_coords: [0.0, 0.0, 0.0, 0.0] });
        buffer.push(Vertex { position: [1.0, 0.0, 0.0, 0.0], char_coords: [0.0, 0.0, 0.0, 0.0] });
        buffer.push(Vertex { position: [1.0, 1.0, 0.0, 0.0], char_coords: [0.0, 0.0, 0.0, 0.0] });
        buffer.push(Vertex { position: [0.0, 1.0, 0.0, 0.0], char_coords: [0.0, 0.0, 0.0, 0.0] });
    }

    buffer

}


// Font Vertex ----------------------------------------------------------------
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 4],
    char_coords: [f32; 4]
}

implement_vertex!(Vertex, position, char_coords);

