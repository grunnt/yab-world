use glm::Mat4;

use crate::video::color::*;
use crate::video::*;
use crate::*;

pub struct SpriteBatcher {
    program: ShaderProgram,
    projection_uniform: Option<UniformLocation>,
    vbo: VBO,
    vao: VAO,
    vertices: Vec<Vertex>,
    texture: TextureAtlas,
}

impl SpriteBatcher {
    pub fn new(
        gl: &glow::Context,
        assets: &Assets,
        vertex_shader_file: &str,
        fragment_shader_file: &str,
        texture: TextureAtlas,
    ) -> Self {
        let program = ShaderProgram::load(
            gl,
            assets,
            vertex_shader_file,
            fragment_shader_file,
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
        Vertex::vertex_attrib_pointers(gl);

        SpriteBatcher {
            program,
            projection_uniform,
            vbo,
            vao,
            vertices: Vec::new(),
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

    pub fn draw(&mut self, gl: &glow::Context, projection: &Mat4) {
        if self.vertices.is_empty() {
            return;
        }

        self.vbo.bind(gl);
        // Increase buffer size if needed
        // TODO fix if needed
        // if self.vertices.len() > self.capacity {
        //     self.capacity = self.vertices.len() * 2;
        //     self.vbo.stream_draw_data_null::<Vertex>(self.capacity);
        // }
        // Upload the vertices
        // unsafe {
        // if let Some(mut buffer) = self
        //     .vbo
        //     .map_buffer_range_write_invalidate::<Vertex>(0, self.vertices.len())
        // {
        //     for i in 0..self.vertices.len() {
        //         *buffer.get_unchecked_mut(i) = self.vertices.get(i).unwrap().clone();
        //     }
        // }
        // }
        self.vbo.stream_draw_data(gl, &self.vertices);
        self.vbo.unbind(gl);

        // Render the sprites
        self.program.set_used(gl);
        if let Some(uniform) = &self.projection_uniform {
            self.program.set_uniform_matrix_4fv(gl, uniform, projection);
        }
        self.vao.bind(gl);
        self.texture.texture().bind_at(gl, 0);
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
