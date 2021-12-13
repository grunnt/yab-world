use glm::Mat4;

use crate::gl;
use crate::video::color::*;
use crate::video::*;
use crate::*;

pub struct PrimitiveRender {
    gl: gl::Gl,
    program: Program,
    projection_uniform: Option<Uniform>,
    vbo: ArrayBuffer,
    vao: VertexArray,
    vertices: Vec<PrimitiveVertex>,
    capacity: usize,
}

impl PrimitiveRender {
    pub fn new(gl: &gl::Gl, assets: &Assets) -> Self {
        let capacity = 256;
        let program = Program::load(
            gl,
            assets,
            vec!["shaders/primitive.vert", "shaders/primitive.frag"],
        )
        .unwrap();
        program.set_used();
        let projection_uniform = program.get_uniform("projection");
        // Vertex array and object
        let vbo = ArrayBuffer::new(gl);
        let vao = VertexArray::new(gl);
        vao.bind();
        vbo.bind();
        PrimitiveVertex::vertex_attrib_pointers(gl);
        // Reserve space in the vbo
        vbo.stream_draw_data_null::<PrimitiveVertex>(capacity);

        PrimitiveRender {
            gl: gl.clone(),
            program,
            projection_uniform,
            vbo,
            vao,
            vertices: Vec::new(),
            capacity,
        }
    }

    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: &ColorRGBA) {
        self.vertices.push(vertex(x1, y1, color));
        self.vertices.push(vertex(x2, y2, color));
    }

    pub fn rectangle(
        &mut self,
        center_x: f32,
        center_y: f32,
        width: f32,
        height: f32,
        angle: f32,
        color: &ColorRGBA,
    ) {
        // Calculate rotated corners
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
        // Create vertices
        let v1 = vertex(x1, y1, color);
        let v2 = vertex(x2, y2, color);
        let v3 = vertex(x3, y3, color);
        let v4 = vertex(x4, y4, color);
        // Lines
        self.vertices.push(v1);
        self.vertices.push(v2);
        self.vertices.push(v2);
        self.vertices.push(v3);
        self.vertices.push(v3);
        self.vertices.push(v4);
        self.vertices.push(v4);
        self.vertices.push(v1);
    }

    pub fn draw(&mut self, projection: &Mat4) {
        if self.vertices.is_empty() {
            return;
        }

        // Upload the vertices
        self.vbo.bind();
        // Increase buffer size if needed
        if self.vertices.len() > self.capacity {
            self.capacity = self.vertices.len() * 2;
            self.vbo
                .stream_draw_data_null::<PrimitiveVertex>(self.capacity);
        }
        // Upload the vertices
        unsafe {
            if let Some(mut buffer) = self
                .vbo
                .map_buffer_range_write_invalidate::<PrimitiveVertex>(0, self.vertices.len())
            {
                for i in 0..self.vertices.len() {
                    *buffer.get_unchecked_mut(i) = self.vertices.get(i).unwrap().clone();
                }
            }
        }
        self.vbo.unbind();

        // Render the lines
        self.program.set_used();
        if let Some(uniform) = &self.projection_uniform {
            uniform.set_uniform_matrix_4fv(projection);
        }
        self.vao.bind();
        unsafe {
            self.gl.Enable(gl::CULL_FACE);
            self.gl.Disable(gl::DEPTH_TEST);
            self.gl.Enable(gl::BLEND);
            self.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            self.gl
                .DrawArrays(gl::LINES, 0, self.vertices.len() as gl::types::GLsizei);
        }

        // Clear the vertex buffer for the next frame
        self.vertices.clear();
    }
}

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct PrimitiveVertex {
    #[location = "0"]
    pub position: data::f32_f32,
    #[location = "1"]
    pub color: data::f32_f32_f32_f32,
}

fn vertex(x: f32, y: f32, color: &ColorRGBA) -> PrimitiveVertex {
    PrimitiveVertex {
        position: (x, y).into(),
        color: (color.r, color.g, color.b, color.a).into(),
    }
}
