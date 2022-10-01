use glm::Mat4;

use crate::video::color::*;
use crate::video::*;
use crate::*;

const VERTEX_SHADER: &str = "
#version 330 core
layout (location = 0) in vec2 Position;
layout (location = 1) in vec2 TexCoords;
layout (location = 2) in vec4 Color;

out vec4 vertColor;
out vec2 texCoords;

uniform mat4 projection;

void main()
{
    vertColor = Color;
    texCoords = TexCoords;
    gl_Position = projection * vec4(Position, 0.0, 1.0);
} 
";

const FRAGMENT_SHADER: &str = "
#version 330 core

uniform sampler2D textureAtlas;

out vec4 fragColor;

in vec4 vertColor;
in vec2 texCoords;

void main()
{
    vec4 color = texture(textureAtlas, texCoords);
    fragColor = vertColor * color;
}
";

pub struct SpriteBatcher {
    program: ShaderProgram,
    projection_uniform: Option<UniformLocation>,
    vbo: VBO,
    vao: VAO,
    vertices: Vec<SpriteVertex>,
    texture_atlas: TextureAtlas,
}

impl SpriteBatcher {
    pub fn new(gl: &glow::Context, texture_atlas: TextureAtlas) -> Self {
        let program = ShaderProgram::from_strings(
            gl,
            VERTEX_SHADER,
            FRAGMENT_SHADER,
            "spritebatcher".to_string(),
        )
        .unwrap();
        program.set_used(gl);
        let projection_uniform = program.get_uniform(gl, "projection");
        if let Some(uniform) = program.get_uniform(gl, "textureAtlas") {
            program.set_uniform_1i(gl, &uniform, 0);
        }
        // Vertex array and object
        let vbo = VBO::new(gl);
        let vao = VAO::new(gl);
        vao.bind(gl);
        vbo.bind(gl);
        SpriteVertex::vertex_attrib_pointers(gl);

        SpriteBatcher {
            program,
            projection_uniform,
            vbo,
            vao,
            vertices: Vec::new(),
            texture_atlas,
        }
    }

    pub fn add(
        &mut self,
        center_x: f32,
        center_y: f32,
        width: f32,
        height: f32,
        angle: f32,
        color: &ColorRGBA,
        texture_id: usize,
    ) {
        let frame = self.texture_atlas.frame(texture_id);
        let hw = width / 2.0;
        let hh = height / 2.0;
        let (x1, y1, x2, y2, x3, y3, x4, y4) = if angle == 0.0 {
            (
                center_x - hw,
                center_y - hh,
                center_x + hw,
                center_y - hh,
                center_x + hw,
                center_y + hh,
                center_x - hw,
                center_y + hh,
            )
        } else {
            let cos = angle.cos();
            let sin = angle.sin();
            (
                center_x - hw * cos + hh * sin,
                center_y - hw * sin - hh * cos,
                center_x + hw * cos + hh * sin,
                center_y + hw * sin - hh * cos,
                center_x + hw * cos - hh * sin,
                center_y + hw * sin + hh * cos,
                center_x - hw * cos - hh * sin,
                center_y - hw * sin + hh * cos,
            )
        };

        let v1 = vertex(x1, y1, frame.x, frame.y, color);
        let v2 = vertex(x2, y2, frame.x + frame.width, frame.y, color);
        let v3 = vertex(x3, y3, frame.x + frame.width, frame.y + frame.height, color);
        let v4 = vertex(x4, y4, frame.x, frame.y + frame.height, color);
        // Triangle 1
        self.vertices.push(v1);
        self.vertices.push(v4);
        self.vertices.push(v2);
        // Triangle 2
        self.vertices.push(v2);
        self.vertices.push(v4);
        self.vertices.push(v3);
    }

    pub fn add_points(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        x4: f32,
        y4: f32,
        color: &ColorRGBA,
        texture_id: usize,
    ) {
        let frame = self.texture_atlas.frame(texture_id);
        let (u1, v1) = (frame.x, frame.y);
        let (u2, v2) = (frame.x + frame.width, frame.y + frame.height);
        let vert1 = vertex(x1, y1, u1, v1, color);
        let vert2 = vertex(x2, y2, u2, v1, color);
        let vert3 = vertex(x3, y3, u2, v2, color);
        let vert4 = vertex(x4, y4, u1, v2, color);
        // Triangle 1
        self.vertices.push(vert1);
        self.vertices.push(vert4);
        self.vertices.push(vert2);
        // Triangle 2
        self.vertices.push(vert2);
        self.vertices.push(vert4);
        self.vertices.push(vert3);
    }

    pub fn draw(&mut self, gl: &glow::Context, projection: &Mat4) {
        if self.vertices.is_empty() {
            return;
        }

        self.vbo.bind(gl);
        self.vbo.stream_draw_data(gl, &self.vertices);
        self.vbo.unbind(gl);

        // Render the sprites
        self.program.set_used(gl);
        if let Some(uniform) = &self.projection_uniform {
            self.program.set_uniform_matrix_4fv(gl, uniform, projection);
        }
        self.vao.bind(gl);
        self.texture_atlas.texture().bind_at(gl, 0);
        unsafe {
            gl.enable(glow::CULL_FACE);
            gl.disable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.draw_arrays(glow::TRIANGLES, 0, self.vertices.len() as i32);
        }

        // Clear the vertex buffer for the next frame
        self.vertices.clear();
    }

    pub fn texture_atlas(&self) -> &TextureAtlas {
        &self.texture_atlas
    }
}

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct SpriteVertex {
    #[location = "0"]
    pub position: data::f32_f32,
    #[location = "1"]
    pub texture_coords: data::f32_f32,
    #[location = "2"]
    pub color: data::f32_f32_f32_f32,
}

fn vertex(x: f32, y: f32, u: f32, v: f32, color: &ColorRGBA) -> SpriteVertex {
    SpriteVertex {
        position: (x, y).into(),
        texture_coords: (u, v).into(),
        color: (color.r, color.g, color.b, color.a).into(),
    }
}
