use failure;
use gamework::glow::*;
use gamework::video::*;
use gamework::*;
use nalgebra_glm::*;
use rand::Rng;

const SSAO_KERNEL_SIZE: usize = 128;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct QuadVertex {
    #[location = "0"]
    pub position: data::f32_f32,
    #[location = "1"]
    pub tex_coords: data::f32_f32,
}

impl QuadVertex {
    pub fn new(x: f32, y: f32, uvx: f32, uvy: f32) -> QuadVertex {
        QuadVertex {
            position: (x, y).into(),
            tex_coords: (uvx, uvy).into(),
        }
    }
}

pub struct DeferredPipeline {
    program: ShaderProgram,
    framebuffer: FBO,
    position_buffer: MyTexture,
    color_buffer: MyTexture,
    normal_buffer: MyTexture,
    lights_buffer: MyTexture,
    view_uniform: Option<UniformLocation>,
    projection_uniform: Option<UniformLocation>,
    light_dir_uniform: Option<UniformLocation>,
    light_col_uniform: Option<UniformLocation>,
    fog_col_uniform: Option<UniformLocation>,
    fog_start_uniform: Option<UniformLocation>,
    fog_end_uniform: Option<UniformLocation>,
    random_texture: Option<MyTexture>,
    ssao_texture_scale_uniform: Option<UniformLocation>,
    _vbo: VBO,
    vao: VAO,
}

impl DeferredPipeline {
    pub fn new(
        width: u32,
        height: u32,
        gl: &glow::Context,
        assets: &Assets,
    ) -> Result<DeferredPipeline, failure::Error> {
        // https://www.gamedev.net/articles/programming/graphics/a-simple-and-practical-approach-to-ssao-r2753/
        let program = ShaderProgram::load(
            gl,
            assets,
            "shaders/deferred.vert",
            "shaders/deferred.frag",
            "deferred".to_string(),
        )?;
        program.set_used(gl);
        if let Some(uniform) = program.get_uniform(gl, "gPosition") {
            program.set_uniform_1i(gl, &uniform, 0);
        }
        if let Some(uniform) = program.get_uniform(gl, "gColor") {
            program.set_uniform_1i(gl, &uniform, 1);
        }
        if let Some(uniform) = program.get_uniform(gl, "gNormal") {
            program.set_uniform_1i(gl, &uniform, 2);
        }
        if let Some(uniform) = program.get_uniform(gl, "gLight") {
            program.set_uniform_1i(gl, &uniform, 3);
        }
        if let Some(uniform) = program.get_uniform(gl, "gRandom") {
            program.set_uniform_1i(gl, &uniform, 4);
        }
        if let Some(uniform) = program.get_uniform(gl, "ssaoKernel") {
            // Based on https://learnopengl.com/Advanced-Lighting/SSAO
            let mut kernel = Vec::new();
            let mut rng = rand::thread_rng();
            for i in 0..SSAO_KERNEL_SIZE {
                let scale = i as f32 / SSAO_KERNEL_SIZE as f32;
                let v = Vec3::new(
                    rng.gen_range(-1.0, 1.0),
                    rng.gen_range(-1.0, 1.0),
                    rng.gen_range(0.0, 1.0),
                )
                .normalize()
                    * scale;
                kernel.push(v.x);
                kernel.push(v.y);
                kernel.push(v.z);
            }
            program.set_uniform_3fv(gl, &uniform, &kernel);
        }
        let view_uniform = program.get_uniform(gl, "View");
        let projection_uniform = program.get_uniform(gl, "Projection");
        let light_dir_uniform = program.get_uniform(gl, "sunLightDirection");
        let light_col_uniform = program.get_uniform(gl, "sunLightColor");
        let fog_col_uniform = program.get_uniform(gl, "fogColor");
        let fog_start_uniform = program.get_uniform(gl, "fogStart");
        let fog_end_uniform = program.get_uniform(gl, "fogEnd");
        let ssao_texture_scale_uniform = program.get_uniform(gl, "ssaoTextureScale");

        let random_texture = Some(
            MyTexture::load(
                &assets.assets_path("textures/random.png"),
                gl,
                TextureFormat::RGBA8,
                TextureWrap::Repeat,
                TextureFilter::Linear,
            )
            .unwrap(),
        );

        // Vertex array
        let mut _vbo = VBO::new(gl);
        // Vertex array object
        let vao = VAO::new(gl);
        vao.bind(gl);
        _vbo.bind(gl);
        let mut quad = Vec::new();
        quad.push(QuadVertex::new(-1.0, 1.0, 0.0, 1.0));
        quad.push(QuadVertex::new(-1.0, -1.0, 0.0, 0.0));
        quad.push(QuadVertex::new(1.0, 1.0, 1.0, 1.0));
        quad.push(QuadVertex::new(1.0, -1.0, 1.0, 0.0));
        _vbo.static_draw_data(gl, &quad);
        QuadVertex::vertex_attrib_pointers(gl);

        // Setup buffers to render to
        let position_buffer = MyTexture::new_uninitialized(
            gl,
            width,
            height,
            TextureFormat::RGBA16F,
            TextureWrap::Clamp,
            TextureFilter::Nearest,
        )?;
        let color_buffer = MyTexture::new_uninitialized(
            gl,
            width,
            height,
            TextureFormat::RGBA16F,
            TextureWrap::None,
            TextureFilter::Nearest,
        )?;
        let normal_buffer = MyTexture::new_uninitialized(
            gl,
            width,
            height,
            TextureFormat::RGBA16F,
            TextureWrap::None,
            TextureFilter::Nearest,
        )?;
        let lights_buffer = MyTexture::new_uninitialized(
            gl,
            width,
            height,
            TextureFormat::RGBA16F,
            TextureWrap::None,
            TextureFilter::Nearest,
        )?;
        let buffers = vec![
            position_buffer.handle,
            color_buffer.handle,
            normal_buffer.handle,
            lights_buffer.handle,
        ];

        // Now create the framebuffer
        let framebuffer = FBO::new(gl, width, height, true, buffers)?;

        Ok(DeferredPipeline {
            program,
            framebuffer,
            position_buffer,
            color_buffer,
            normal_buffer,
            lights_buffer,
            view_uniform,
            projection_uniform,
            light_dir_uniform,
            light_col_uniform,
            fog_col_uniform,
            fog_start_uniform,
            fog_end_uniform,
            random_texture,
            ssao_texture_scale_uniform,
            _vbo,
            vao,
        })
    }

    pub fn bind(&self, gl: &glow::Context) {
        self.framebuffer.bind(gl);
    }

    pub fn unbind(&self, gl: &glow::Context) {
        self.framebuffer.unbind(gl);
    }

    pub fn render_to_screen(
        &mut self,
        gl: &glow::Context,
        camera: &PerspectiveCamera,
        width: u32,
        height: u32,
        sun_dir: &Vec3,
        sun_col: &Vec3,
        fog_col: &Vec3,
        fog_start: f32,
        fog_end: f32,
    ) {
        self.framebuffer.copy_depth_to_default(gl);
        self.program.set_used(gl);
        if let Some(uniform) = &self.view_uniform {
            self.program
                .set_uniform_matrix_4fv(gl, &uniform, camera.get_view());
        }
        if let Some(uniform) = &self.projection_uniform {
            self.program
                .set_uniform_matrix_4fv(gl, &uniform, camera.get_projection());
        }
        if let Some(uniform) = &self.light_dir_uniform {
            self.program.set_uniform_3f(gl, &uniform, sun_dir);
        }
        if let Some(uniform) = &self.light_col_uniform {
            self.program.set_uniform_3f(gl, &uniform, sun_col);
        }
        if let Some(uniform) = &self.fog_col_uniform {
            self.program.set_uniform_3f(gl, &uniform, fog_col);
        }
        if let Some(uniform) = &self.fog_start_uniform {
            self.program.set_uniform_1f(gl, &uniform, fog_start);
        }
        if let Some(uniform) = &self.fog_end_uniform {
            self.program.set_uniform_1f(gl, &uniform, fog_end);
        }
        if let Some(uniform) = &self.ssao_texture_scale_uniform {
            let texture_size = 64.0;
            let texture_ratio =
                Vec2::new(width as f32 / texture_size, height as f32 / texture_size);
            self.program.set_uniform_2f(gl, &uniform, &texture_ratio);
        }
        self.vao.bind(gl);
        unsafe {
            gl.disable(glow::CULL_FACE);
            gl.disable(glow::DEPTH_TEST);
            gl.disable(glow::BLEND);
            gl.enable(glow::FRAMEBUFFER_SRGB);
            self.position_buffer.bind_at(gl, 0);
            self.color_buffer.bind_at(gl, 1);
            self.normal_buffer.bind_at(gl, 2);
            self.lights_buffer.bind_at(gl, 3);
            if let Some(random_texture) = &self.random_texture {
                random_texture.bind_at(gl, 4);
            }
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            gl.disable(glow::FRAMEBUFFER_SRGB);
        }
    }
}
