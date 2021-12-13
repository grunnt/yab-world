use gamework::gl;
use gamework::video::*;
use gamework::*;
use nalgebra_glm::*;

pub const SKYDOME_SCALE: f32 = 1000.0;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct SkyDomeVertex {
    #[location = "0"]
    pub position: data::f32_f32_f32,
    #[location = "1"]
    pub normal: data::f32_f32_f32,
}

pub struct SkyDome {
    gl: gl::Gl,
    program: Program,
    _vbo: ArrayBuffer,
    vao: VertexArray,
    vertex_count: gl::types::GLsizei,
    model_uniform: Option<Uniform>,
    view_uniform: Option<Uniform>,
    projection_uniform: Option<Uniform>,
    fog_color_uniform: Option<Uniform>,
    sky_color_uniform: Option<Uniform>,
    light_dir_uniform: Option<Uniform>,
    light_col_uniform: Option<Uniform>,
    dither_texture: Option<Texture>,
}

impl SkyDome {
    pub fn new(gl: &gl::Gl, assets: &Assets) -> SkyDome {
        let program = Program::load(
            gl,
            assets,
            vec!["shaders/skydome.vert", "shaders/skydome.frag"],
        )
        .unwrap();
        program.set_used();
        let model_uniform = program.get_uniform("Model");
        let view_uniform = program.get_uniform("View");
        let projection_uniform = program.get_uniform("Projection");
        let fog_color_uniform = program.get_uniform("fogColor");
        let sky_color_uniform = program.get_uniform("skyColor");
        let light_dir_uniform = program.get_uniform("sunLightDirection");
        let light_col_uniform = program.get_uniform("sunColor");
        if let Some(uniform) = program.get_uniform("ditherTexture") {
            uniform.set_uniform_1i(0);
        }
        let dither_texture = Some(
            Texture::load(
                &assets.assets_path("textures/bayer_dither.png"),
                gl,
                TextureFormat::RGBA8,
                TextureWrap::Repeat,
                TextureFilter::Nearest,
            )
            .unwrap(),
        );

        // Currently model is a full sphere, we might change this to hemisphere for performance
        let (models, _) = tobj::load_obj(assets.assets_path("objects/skydome.obj"), true)
            .expect("Failed to load object file");
        assert!(models.len() == 1);
        let model = models.get(0).unwrap();
        let mesh = &model.mesh;

        let mut vertices = Vec::new();
        for index in &mesh.indices {
            let i = *index as usize;
            vertices.push(SkyDomeVertex {
                position: (
                    mesh.positions[3 * i] * SKYDOME_SCALE,
                    mesh.positions[3 * i + 1] * SKYDOME_SCALE,
                    mesh.positions[3 * i + 2] * SKYDOME_SCALE,
                )
                    .into(),
                normal: (
                    mesh.normals[3 * i],
                    mesh.normals[3 * i + 1],
                    mesh.normals[3 * i + 2],
                )
                    .into(),
            });
        }

        // Vertex array
        let mut _vbo = ArrayBuffer::new(gl);
        let vao = VertexArray::new(gl);

        vao.bind();
        _vbo.bind();
        _vbo.static_draw_data(&vertices, false);
        SkyDomeVertex::vertex_attrib_pointers(gl);

        SkyDome {
            gl: gl.clone(),
            program,
            _vbo,
            vao,
            vertex_count: vertices.len() as gl::types::GLsizei,
            model_uniform,
            view_uniform,
            projection_uniform,
            fog_color_uniform,
            sky_color_uniform,
            light_dir_uniform,
            light_col_uniform,
            dither_texture,
        }
    }

    pub fn render(
        &self,
        model: Mat4,
        camera: &PerspectiveCamera,
        fog_color: Vec3,
        sky_color: Vec3,
        sun_dir: &Vec3,
        sun_col: &Vec3,
    ) {
        self.program.set_used();
        if let Some(uniform) = &self.model_uniform {
            uniform.set_uniform_matrix_4fv(&model);
        }
        if let Some(uniform) = &self.view_uniform {
            uniform.set_uniform_matrix_4fv(camera.get_view());
        }
        if let Some(uniform) = &self.projection_uniform {
            uniform.set_uniform_matrix_4fv(camera.get_projection());
        }
        if let Some(uniform) = &self.fog_color_uniform {
            uniform.set_uniform_3f(&fog_color);
        }
        if let Some(uniform) = &self.sky_color_uniform {
            uniform.set_uniform_3f(&sky_color);
        }
        if let Some(uniform) = &self.light_dir_uniform {
            uniform.set_uniform_3f(&(*sun_dir * -1.0));
        }
        if let Some(uniform) = &self.light_col_uniform {
            uniform.set_uniform_3f(sun_col);
        }
        if let Some(dither_texture) = &self.dither_texture {
            dither_texture.bind_at(0);
        }
        self.vao.bind();
        unsafe {
            self.gl.Enable(gl::CULL_FACE);
            self.gl.Enable(gl::DEPTH_TEST);
            self.gl.Disable(gl::BLEND);
        }
        self.vao.bind();
        unsafe {
            self.gl.DrawArrays(gl::TRIANGLES, 0, self.vertex_count);
        }
    }
}
