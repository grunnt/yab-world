#![allow(dead_code)]
use gamework::gl;
use gamework::video::data;
use gamework::video::*;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Vertex {
    #[location = "0"]
    pub position: data::f32_f32_f32,
    #[location = "1"]
    pub color: data::f32_f32_f32,
    #[location = "2"]
    pub normal: data::f32_f32_f32,
    #[location = "3"]
    pub lights: data::f32_f32_f32,
}

pub struct Mesh {
    pub _vbo: ArrayBuffer,
    pub vao: VertexArray,
    pub vertex_count: gl::types::GLsizei,
}

impl Mesh {
    pub fn new(gl: &gl::Gl, vertices: &Vec<Vertex>) -> Option<Mesh> {
        if vertices.len() == 0 {
            None
        } else {
            // Vertex array
            let mut _vbo = ArrayBuffer::new(gl);

            // Vertex array object
            let vao = VertexArray::new(gl);

            vao.bind();
            _vbo.bind();
            _vbo.static_draw_data(vertices, false);
            Vertex::vertex_attrib_pointers(gl);

            Some(Mesh {
                _vbo,
                vao,
                vertex_count: vertices.len() as gl::types::GLsizei,
            })
        }
    }

    pub fn render(&self, gl: &gl::Gl) {
        self.vao.bind();
        unsafe {
            gl.DrawArrays(gl::TRIANGLES, 0, self.vertex_count);
        }
        //self.vao.unbind();
    }

    pub fn render_lines(&self, gl: &gl::Gl) {
        self.vao.bind();
        unsafe {
            gl.DrawArrays(gl::LINES, 0, self.vertex_count);
        }
        //self.vao.unbind();
    }
}

pub struct IndexedMesh {
    pub _vbo: ArrayBuffer,
    pub _ebo: ElementArrayBuffer,
    pub vao: VertexArray,
    pub index_count: gl::types::GLsizei,
}

impl IndexedMesh {
    pub fn new(gl: &gl::Gl, vertices: &Vec<Vertex>, indices: &Vec<u32>) -> Option<IndexedMesh> {
        if vertices.len() == 0 || indices.len() == 0 {
            None
        } else {
            // Vertex array
            let mut _vbo = ArrayBuffer::new(gl);
            _vbo.bind();
            _vbo.static_draw_data(vertices, false);
            _vbo.unbind();

            // Vertex element buffer
            let mut _ebo = ElementArrayBuffer::new(gl);
            _ebo.bind();
            _ebo.static_draw_data(indices, false);
            _ebo.unbind();

            // Vertex array object
            let vao = VertexArray::new(gl);

            vao.bind();
            _ebo.bind();
            _vbo.bind();
            Vertex::vertex_attrib_pointers(gl);
            vao.unbind();
            _vbo.unbind();
            _ebo.unbind();

            Some(IndexedMesh {
                _vbo,
                _ebo,
                vao,
                index_count: indices.len() as gl::types::GLsizei,
            })
        }
    }

    pub fn render(&self, gl: &gl::Gl) {
        self.vao.bind();
        unsafe {
            gl.DrawElements(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_INT,
                ::std::ptr::null(),
            );
        }
    }

    pub fn render_lines(&self, gl: &gl::Gl) {
        self.vao.bind();
        unsafe {
            gl.DrawElements(
                gl::LINES,
                self.index_count,
                gl::UNSIGNED_INT,
                ::std::ptr::null(),
            );
        }
    }
}
