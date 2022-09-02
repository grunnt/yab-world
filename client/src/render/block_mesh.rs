#![allow(dead_code)]
use gamework::glow;
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
    pub vbo: VBO,
    pub vao: VAO,
    pub vertex_count: i32,
    pub creation_instant: Instant,
    pub animate: bool,
}

impl BlockMesh {
    pub fn new(
        gl: &glow::Context,
        vertices: &Vec<BlockVertex>,
        animate: bool,
    ) -> Option<BlockMesh> {
        if vertices.len() == 0 {
            None
        } else {
            // Vertex array
            let vbo = VBO::new(gl);

            // Vertex array object
            let vao = VAO::new(gl);

            vao.bind(gl);
            vbo.bind(gl);
            vbo.static_draw_data(gl, vertices);
            BlockVertex::vertex_attrib_pointers(gl);

            let creation_instant = Instant::now();

            Some(BlockMesh {
                vbo,
                vao,
                vertex_count: vertices.len() as i32,
                creation_instant,
                animate,
            })
        }
    }

    pub fn render(&mut self, gl: &glow::Context) {
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
}
