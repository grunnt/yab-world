#![allow(dead_code)]
use gamework::gl;
use gamework::video::*;
use std::time::Instant;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct BlockVertex {
    #[location = "0"]
    pub position: data::f32_f32_f32,
    #[location = "1"]
    pub texture: data::f32_f32_f32,
    #[location = "2"]
    pub normal: data::u8_,
    #[location = "3"]
    pub light: data::u8_,
}

impl BlockVertex {
    pub fn new(
        x: f32,
        y: f32,
        z: f32,
        texture_x: f32,
        texture_y: f32,
        texture_layer: f32,
        blocklight: u8,
        normal: u8,
    ) -> BlockVertex {
        BlockVertex {
            position: (x, y, z).into(),
            texture: (texture_x, texture_y, texture_layer).into(),
            normal: normal.into(),
            light: blocklight.into(),
        }
    }
}

pub struct BlockMesh {
    pub vbo: ArrayBuffer,
    pub vao: VertexArray,
    pub vertex_count: gl::types::GLsizei,
    pub creation_instant: Instant,
    pub animate: bool,
}

impl BlockMesh {
    pub fn new(
        gl: &gl::Gl,
        vertices: &Vec<BlockVertex>,
        fence: bool,
        animate: bool,
    ) -> Option<BlockMesh> {
        if vertices.len() == 0 {
            None
        } else {
            // Vertex array
            let mut vbo = ArrayBuffer::new(gl);

            // Vertex array object
            let vao = VertexArray::new(gl);

            vao.bind();
            vbo.bind();
            vbo.static_draw_data(vertices, fence);
            BlockVertex::vertex_attrib_pointers(gl);

            let creation_instant = Instant::now();

            Some(BlockMesh {
                vbo,
                vao,
                vertex_count: vertices.len() as gl::types::GLsizei,
                creation_instant,
                animate,
            })
        }
    }

    pub fn render(&mut self, gl: &gl::Gl) {
        if self.vbo.passed_fence() {
            self.vao.bind();
            unsafe {
                gl.DrawArrays(gl::TRIANGLES, 0, self.vertex_count);
            }
        }
    }

    pub fn render_lines(&self, gl: &gl::Gl) {
        self.vao.bind();
        unsafe {
            gl.DrawArrays(gl::LINES, 0, self.vertex_count);
        }
    }
}
