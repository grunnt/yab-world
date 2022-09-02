use gamework::glow;
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
    program: ShaderProgram,
    _vbo: VBO,
    vao: VAO,
    vertex_count: i32,
    model_uniform: Option<UniformLocation>,
    view_uniform: Option<UniformLocation>,
    projection_uniform: Option<UniformLocation>,
    fog_color_uniform: Option<UniformLocation>,
    sky_color_uniform: Option<UniformLocation>,
    light_dir_uniform: Option<UniformLocation>,
    light_col_uniform: Option<UniformLocation>,
}

impl SkyDome {
    pub fn new(gl: &glow::Context, assets: &Assets) -> SkyDome {
        let program = ShaderProgram::load(
            gl,
            assets,
            "shaders/skydome.vert",
            "shaders/skydome.frag",
            "skydome".to_string(),
        )
        .unwrap();
        program.set_used(gl);
        let model_uniform = program.get_uniform(gl, "Model");
        let view_uniform = program.get_uniform(gl, "View");
        let projection_uniform = program.get_uniform(gl, "Projection");
        let fog_color_uniform = program.get_uniform(gl, "fogColor");
        let sky_color_uniform = program.get_uniform(gl, "skyColor");
        let light_dir_uniform = program.get_uniform(gl, "sunLightDirection");
        let light_col_uniform = program.get_uniform(gl, "sunColor");

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
        let mut _vbo = VBO::new(gl);
        let vao = VAO::new(gl);

        vao.bind(gl);
        _vbo.bind(gl);
        _vbo.static_draw_data(gl, &vertices);
        SkyDomeVertex::vertex_attrib_pointers(gl);

        SkyDome {
            program,
            _vbo,
            vao,
            vertex_count: vertices.len() as i32,
            model_uniform,
            view_uniform,
            projection_uniform,
            fog_color_uniform,
            sky_color_uniform,
            light_dir_uniform,
            light_col_uniform,
        }
    }

    pub fn render(
        &self,
        gl: &glow::Context,
        model: Mat4,
        camera: &PerspectiveCamera,
        fog_color: Vec3,
        sky_color: Vec3,
        sun_dir: &Vec3,
        sun_col: &Vec3,
    ) {
        self.program.set_used(gl);
        if let Some(uniform) = &self.model_uniform {
            self.program.set_uniform_matrix_4fv(gl, &uniform, &model);
        }
        if let Some(uniform) = &self.view_uniform {
            self.program
                .set_uniform_matrix_4fv(gl, &uniform, camera.get_view());
        }
        if let Some(uniform) = &self.projection_uniform {
            self.program
                .set_uniform_matrix_4fv(gl, &uniform, camera.get_projection());
        }
        if let Some(uniform) = &self.fog_color_uniform {
            self.program.set_uniform_3f(gl, &uniform, &fog_color);
        }
        if let Some(uniform) = &self.sky_color_uniform {
            self.program.set_uniform_3f(gl, &uniform, &sky_color);
        }
        if let Some(uniform) = &self.light_dir_uniform {
            self.program
                .set_uniform_3f(gl, &uniform, &(*sun_dir * -1.0));
        }
        if let Some(uniform) = &self.light_col_uniform {
            self.program.set_uniform_3f(gl, &uniform, sun_col);
        }
        self.vao.bind(gl);
        unsafe {
            gl.enable(glow::CULL_FACE);
            gl.enable(glow::DEPTH_TEST);
            gl.disable(glow::BLEND);
            gl.enable(glow::FRAMEBUFFER_SRGB);
        }
        self.vao.bind(gl);
        unsafe {
            gl.draw_arrays(glow::TRIANGLES, 0, self.vertex_count);
            gl.disable(glow::FRAMEBUFFER_SRGB);
        }
    }
}
