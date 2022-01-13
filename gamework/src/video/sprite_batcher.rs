use glm::Mat4;

use crate::gl;
use crate::video::color::*;
use crate::video::*;
use crate::*;

pub struct SpriteBatcher {
    gl: gl::Gl,
    program: Program,
    projection_uniform: Option<Uniform>,
    vbo: ArrayBuffer,
    vao: VertexArray,
    vertices: Vec<Vertex>,
    capacity: usize,
    texture: TextureAtlas,
}

impl SpriteBatcher {
    pub fn new(
        gl: &gl::Gl,
        assets: &Assets,
        shader_files: Vec<&str>,
        texture: TextureAtlas,
    ) -> Self {
        let capacity = 512;
        let program = Program::load(gl, assets, shader_files, "spritebatcher".to_string()).unwrap();
        program.set_used();
        let projection_uniform = program.get_uniform("projection");
        if let Some(uniform) = program.get_uniform("textureAtlas") {
            uniform.set_uniform_1i(0);
        }
        // Vertex array and object
        let vbo = ArrayBuffer::new(gl);
        let vao = VertexArray::new(gl);
        vao.bind();
        vbo.bind();
        Vertex::vertex_attrib_pointers(gl);
        // Reserve space in the vbo
        vbo.stream_draw_data_null::<Vertex>(capacity);

        SpriteBatcher {
            gl: gl.clone(),
            program,
            projection_uniform,
            vbo,
            vao,
            vertices: Vec::new(),
            capacity,
            texture,
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
        let frame = self.texture.frame(texture_id);
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
        let frame = self.texture.frame(texture_id);
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

    pub fn draw(&mut self, projection: &Mat4) {
        if self.vertices.is_empty() {
            return;
        }

        self.vbo.bind();
        // Increase buffer size if needed
        if self.vertices.len() > self.capacity {
            self.capacity = self.vertices.len() * 2;
            self.vbo.stream_draw_data_null::<Vertex>(self.capacity);
        }
        // Upload the vertices
        unsafe {
            if let Some(mut buffer) = self
                .vbo
                .map_buffer_range_write_invalidate::<Vertex>(0, self.vertices.len())
            {
                for i in 0..self.vertices.len() {
                    *buffer.get_unchecked_mut(i) = self.vertices.get(i).unwrap().clone();
                }
            }
        }
        self.vbo.unbind();

        // Render the sprites
        self.program.set_used();
        if let Some(uniform) = &self.projection_uniform {
            uniform.set_uniform_matrix_4fv(projection);
        }
        self.vao.bind();
        self.texture.texture().bind_at(0);
        unsafe {
            self.gl.Enable(gl::CULL_FACE);
            self.gl.Disable(gl::DEPTH_TEST);
            self.gl.Enable(gl::BLEND);
            self.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            self.gl
                .DrawArrays(gl::TRIANGLES, 0, self.vertices.len() as gl::types::GLsizei);
        }

        // Clear the vertex buffer for the next frame
        self.vertices.clear();
    }

    pub fn texture_atlas(&self) -> &TextureAtlas {
        &self.texture
    }
}

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Vertex {
    #[location = "0"]
    pub position: data::f32_f32,
    #[location = "1"]
    pub texture_coords: data::f32_f32,
    #[location = "2"]
    pub color: data::f32_f32_f32_f32,
}

fn vertex(x: f32, y: f32, u: f32, v: f32, color: &ColorRGBA) -> Vertex {
    Vertex {
        position: (x, y).into(),
        texture_coords: (u, v).into(),
        color: (color.r, color.g, color.b, color.a).into(),
    }
}
