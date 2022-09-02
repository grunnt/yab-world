#![allow(dead_code)]

use crate::video::buffer;
use glow::*;
use render_derive::*;

use super::data;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Vertex {
    #[location = "0"]
    pub position: data::f32_f32_f32,
    #[location = "1"]
    pub color: data::f32_f32_f32,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32) -> Vertex {
        Vertex {
            position: (x, y, z).into(),
            color: (r, g, b).into(),
        }
    }
}

pub struct Mesh {
    vbo: buffer::VBO,
    vao: buffer::VAO,
    vertex_count: i32,
}

impl Mesh {
    pub fn new(gl: &glow::Context, vertices: &Vec<Vertex>) -> Mesh {
        // Vertex buffer object
        let vbo = buffer::VBO::new(gl);

        // Vertex array object
        let vao = buffer::VAO::new(gl);

        // Load the vertices and bind the attributes
        vao.bind(gl);
        vbo.bind(gl);
        vbo.static_draw_data(gl, vertices);
        Vertex::vertex_attrib_pointers(gl);

        Mesh {
            vbo,
            vao,
            vertex_count: vertices.len() as i32,
        }
    }

    pub fn render_triangles(&self, gl: &glow::Context) {
        self.vao.bind(gl);
        unsafe {
            gl.draw_arrays(glow::TRIANGLES, 0, self.vertex_count);
        }
    }

    pub fn render_lines(&self, gl: &glow::Context) {
        self.vao.bind(gl);
        unsafe {
            gl.draw_arrays(glow::LINES, 0, self.vertex_count);
        }
    }

    pub fn render_points(&mut self, gl: &glow::Context) {
        self.vao.bind(gl);
        unsafe {
            gl.draw_arrays(glow::POINTS, 0, self.vertex_count);
        }
    }

    /// Drop the mesh buffers. This is not needed on shutdown, as it will be cleaned up along with the OpenGL context.
    pub fn drop(&self, gl: &glow::Context) {
        self.vao.drop(gl);
        self.vbo.drop(gl);
    }
}
