#![allow(dead_code)]

use crate::video::buffer;
use crate::video::data;
use gl;
use render_derive::VertexAttribPointers;

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
    gl: gl::Gl,
    vbo: buffer::ArrayBuffer,
    vao: buffer::VertexArray,
    vertex_count: gl::types::GLsizei,
}

impl Mesh {
    pub fn new(gl: &gl::Gl, vertices: &Vec<Vertex>) -> Mesh {
        // Vertex buffer object
        let mut vbo = buffer::ArrayBuffer::new(gl);

        // Vertex array object
        let vao = buffer::VertexArray::new(gl);

        // Load the vertices and bind the attributes
        vao.bind();
        vbo.bind();
        vbo.static_draw_data(vertices, false);
        Vertex::vertex_attrib_pointers(gl);

        Mesh {
            gl: gl.clone(),
            vbo,
            vao,
            vertex_count: vertices.len() as gl::types::GLsizei,
        }
    }

    pub fn render_triangles(&mut self) {
        self.vao.bind();
        unsafe {
            self.gl.DrawArrays(gl::TRIANGLES, 0, self.vertex_count);
        }
    }

    pub fn render_lines(&mut self) {
        self.vao.bind();
        unsafe {
            self.gl.DrawArrays(gl::LINES, 0, self.vertex_count);
        }
    }

    pub fn render_points(&mut self) {
        self.vao.bind();
        unsafe {
            self.gl.DrawArrays(gl::POINTS, 0, self.vertex_count);
        }
    }
}
