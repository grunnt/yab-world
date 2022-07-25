use gamework::video::color::ColorRGBA;
use gamework::video::*;
use gamework::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Rect {
    pub x: i16,
    pub y: i16,
    pub width: i16,
    pub height: i16,
    pub focus: bool,
}

pub struct GuiRenderer {
    gl: gl::Gl,
    program: Program,
    projection_uniform: Option<Uniform>,
    frame_meshes: Vec<HashSet<Rect>>,
    meshes: Vec<HashMap<Rect, RectMesh>>,
    corner_size: f32,
    text: Text,
    sprite_batcher: SpriteBatcher,
    primitive_render: PrimitiveRender,
    ui_camera: OrthographicCamera,
}

impl GuiRenderer {
    pub fn new(gl: &gl::Gl, assets: &Assets, width: u32, height: u32) -> Self {
        let program = Program::load(
            gl,
            assets,
            vec!["shaders/primitive.vert", "shaders/primitive.frag"],
            "primitive".to_string(),
        )
        .unwrap();
        program.set_used();
        let projection_uniform = program.get_uniform("projection");

        let mut meshes = Vec::new();
        for _ in 0..256 {
            meshes.push(HashMap::new());
        }

        let text = Text::new(&assets, &gl, vec![("font.ttf", 28.0), ("font.ttf", 14.0)]);

        let texture = TextureAtlas::load(
            &assets.assets_path("atlas/blocks.png"),
            &assets.assets_path("atlas/blocks.json"),
            gl,
        );

        let sprite_batcher = SpriteBatcher::new(
            gl,
            assets,
            vec!["shaders/sprite.vert", "shaders/sprite.frag"],
            texture,
        );

        let primitive_render = PrimitiveRender::new(gl, assets);

        GuiRenderer {
            gl: gl.clone(),
            program,
            projection_uniform,
            frame_meshes: vec![HashSet::new(); 256],
            meshes,
            corner_size: 3.0,
            text,
            sprite_batcher,
            primitive_render,
            ui_camera: OrthographicCamera::new(width, height),
        }
    }

    pub fn ui_camera(&self) -> &OrthographicCamera {
        &self.ui_camera
    }

    pub fn resize(&mut self, size: Size) {
        self.ui_camera
            .set_size(size.width as u32, size.height as u32);
    }

    pub fn text(&self) -> &Text {
        &self.text
    }

    pub fn text_mut(&mut self) -> &mut Text {
        &mut self.text
    }

    pub fn render_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        focus: bool,
        layer: u8,
        color: &ColorRGBA,
    ) {
        let rect = Rect {
            x: x as i16,
            y: y as i16,
            width: width as i16,
            height: height as i16,
            focus,
        };
        if !self.frame_meshes[layer as usize].contains(&rect) {
            self.frame_meshes[layer as usize].insert(rect.clone());
            if !self.meshes[layer as usize].contains_key(&rect) {
                // Create the rectangle mesh
                let x = rect.x as f32;
                let y = rect.y as f32;
                let w = rect.width as f32;
                let h = rect.height as f32;
                let center_color = (color.r, color.g, color.b, 1.0).into();
                let color = (color.r, color.g, color.b, color.a).into();
                let mut vertices = Vec::new();
                let cx = if w < 2.0 * self.corner_size {
                    w / 2.0
                } else {
                    self.corner_size
                };
                let cy = if h < 2.0 * self.corner_size {
                    h / 2.0
                } else {
                    self.corner_size
                };
                self.corner_size;
                // Build a triangle fan
                vertices.push(RectVertex {
                    position: (x + w / 2.0, y + h / 2.0).into(),
                    color: center_color,
                });
                vertices.push(RectVertex {
                    position: (x + w - cx, y + h).into(),
                    color,
                });
                vertices.push(RectVertex {
                    position: (x + w, y + h - cy).into(),
                    color,
                });
                vertices.push(RectVertex {
                    position: (x + w, y + cy).into(),
                    color,
                });
                vertices.push(RectVertex {
                    position: (x + w - cx, y).into(),
                    color,
                });
                vertices.push(RectVertex {
                    position: (x + cx + 1.0, y).into(),
                    color,
                });
                vertices.push(RectVertex {
                    position: (x, y + cy).into(),
                    color,
                });
                vertices.push(RectVertex {
                    position: (x, y + h - cy).into(),
                    color,
                });
                vertices.push(RectVertex {
                    position: (x + cx + 1.0, y + h).into(),
                    color,
                });
                vertices.push(RectVertex {
                    position: (x + w - cx, y + h).into(),
                    color,
                });
                let mesh = RectMesh::new(&self.gl, &vertices);

                // Store the mesh
                self.meshes[layer as usize].insert(rect, mesh);
            }
        }
    }

    pub fn sprite_batcher_mut(&mut self) -> &mut SpriteBatcher {
        &mut self.sprite_batcher
    }

    pub fn primitive_render_mut(&mut self) -> &mut PrimitiveRender {
        &mut self.primitive_render
    }

    pub fn render(&mut self) {
        // Remove meshes not painted this frame
        let projection = self.ui_camera.get_projection().clone();
        let frame_meshes = &self.frame_meshes;
        for l in 0..256 {
            self.meshes[l].retain(|r, _| frame_meshes[l].contains(r));
        }
        self.program.set_used();
        if let Some(uniform) = &self.projection_uniform {
            uniform.set_uniform_matrix_4fv(&projection);
        }
        unsafe {
            self.gl.Enable(gl::CULL_FACE);
            self.gl.Disable(gl::DEPTH_TEST);
            self.gl.Enable(gl::BLEND);
            self.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
        // Render rectangles in layers
        for l in 0..256 {
            for mesh in self.meshes[l].values() {
                mesh.render();
            }
            self.frame_meshes[l].clear();
        }

        self.sprite_batcher_mut().draw(&projection);

        self.primitive_render.draw(&projection);

        self.text.draw(self.ui_camera.get_projection());
    }
}

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct RectVertex {
    #[location = "0"]
    pub position: data::f32_f32,
    #[location = "1"]
    pub color: data::f32_f32_f32_f32,
}

pub struct RectMesh {
    gl: gl::Gl,
    _vbo: ArrayBuffer,
    vao: VertexArray,
    vertex_count: gl::types::GLsizei,
}

impl RectMesh {
    pub fn new(gl: &gl::Gl, vertices: &Vec<RectVertex>) -> Self {
        // Vertex buffer object
        let mut _vbo = ArrayBuffer::new(gl);

        // Vertex array object
        let vao = VertexArray::new(gl);

        // Load the vertices and bind the attributes
        vao.bind();
        _vbo.bind();
        _vbo.static_draw_data(vertices, false);
        RectVertex::vertex_attrib_pointers(gl);

        RectMesh {
            gl: gl.clone(),
            _vbo,
            vao,
            vertex_count: vertices.len() as gl::types::GLsizei,
        }
    }

    pub fn render(&self) {
        self.vao.bind();
        unsafe {
            self.gl.DrawArrays(gl::TRIANGLE_FAN, 0, self.vertex_count);
        }
    }
}
