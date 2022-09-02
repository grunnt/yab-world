use glm::Mat4;

use crate::video::color::*;
use crate::video::*;
use crate::*;
use glow::*;

pub struct PrimitiveRender {
    program: ShaderProgram,
    projection_uniform: Option<UniformLocation>,
    vbo: VBO,
    vao: VAO,
    vertices: Vec<PrimitiveVertex>,
}

impl PrimitiveRender {
    pub fn new(gl: &glow::Context, assets: &Assets) -> Self {
        let program = ShaderProgram::load(
            gl,
            assets,
            "shaders/primitive.vert",
            "shaders/primitive.frag",
            "primitive".to_string(),
        )
        .unwrap();
        program.set_used(gl);
        let projection_uniform = program.get_uniform(gl, "projection");
        // Vertex array and object
        let vbo = VBO::new(gl);
        let vao = VAO::new(gl);
        vao.bind(gl);
        vbo.bind(gl);
        PrimitiveVertex::vertex_attrib_pointers(gl);

        PrimitiveRender {
            program,
            projection_uniform,
            vbo,
            vao,
            vertices: Vec::new(),
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

    pub fn draw(&mut self, gl: &glow::Context, projection: &Mat4) {
        if self.vertices.is_empty() {
            return;
        }

        // Upload the vertices
        self.vbo.bind(gl);
        self.vbo.stream_draw_data(gl, &self.vertices);
        // TODO fix if needed
        // // Increase buffer size if needed
        // if self.vertices.len() > self.capacity {
        //     self.capacity = self.vertices.len() * 2;
        //     self.vbo
        //         .stream_draw_data_null::<PrimitiveVertex>(self.capacity);
        // }
        // // Upload the vertices
        // unsafe {
        //     if let Some(mut buffer) = self
        //         .vbo
        //         .map_buffer_range_write_invalidate::<PrimitiveVertex>(0, self.vertices.len())
        //     {
        //         for i in 0..self.vertices.len() {
        //             *buffer.get_unchecked_mut(i) = self.vertices.get(i).unwrap().clone();
        //         }
        //     }
        // }
        self.vbo.unbind(gl);

        // Render the lines
        self.program.set_used(gl);
        if let Some(uniform) = &self.projection_uniform {
            self.program.set_uniform_matrix_4fv(gl, uniform, projection);
        }
        self.vao.bind(gl);
        unsafe {
            gl.enable(glow::CULL_FACE);
            gl.disable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.draw_arrays(glow::LINES, 0, self.vertices.len() as i32);
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
