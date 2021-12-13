use failure;
use gamework::video::*;
use gamework::*;
use gl;
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
    gl: gl::Gl,
    program: Program,
    framebuffer: FrameBuffer,
    position_buffer: Texture,
    color_buffer: Texture,
    normal_buffer: Texture,
    lights_buffer: Texture,
    view_uniform: Option<Uniform>,
    projection_uniform: Option<Uniform>,
    ambient_col_uniform: Option<Uniform>,
    light_dir_uniform: Option<Uniform>,
    light_col_uniform: Option<Uniform>,
    fog_col_uniform: Option<Uniform>,
    fog_start_uniform: Option<Uniform>,
    fog_end_uniform: Option<Uniform>,
    random_texture: Option<Texture>,
    ssao_texture_scale_uniform: Option<Uniform>,
    _vbo: ArrayBuffer,
    vao: VertexArray,
}

impl DeferredPipeline {
    pub fn new(
        width: u32,
        height: u32,
        gl: &gl::Gl,
        assets: &Assets,
    ) -> Result<DeferredPipeline, failure::Error> {
        // https://www.gamedev.net/articles/programming/graphics/a-simple-and-practical-approach-to-ssao-r2753/
        let program = Program::load(
            gl,
            assets,
            vec!["shaders/deferred.vert", "shaders/deferred.frag"],
        )?;
        program.set_used();
        if let Some(uniform) = program.get_uniform("gPosition") {
            uniform.set_uniform_1i(0);
        }
        if let Some(uniform) = program.get_uniform("gColor") {
            uniform.set_uniform_1i(1);
        }
        if let Some(uniform) = program.get_uniform("gNormal") {
            uniform.set_uniform_1i(2);
        }
        if let Some(uniform) = program.get_uniform("gLight") {
            uniform.set_uniform_1i(3);
        }
        if let Some(uniform) = program.get_uniform("gRandom") {
            uniform.set_uniform_1i(4);
        }
        if let Some(uniform) = program.get_uniform("ssaoKernel") {
            let mut kernel = Vec::new();
            let mut rng = rand::thread_rng();
            for i in 0..SSAO_KERNEL_SIZE {
                let scale = i as f32 / SSAO_KERNEL_SIZE as f32;
                let x: f32 = rng.gen();
                let y: f32 = rng.gen();
                let z: f32 = rng.gen();
                let mut v = Vec3::new(x - 0.5, y - 0.5, z - 0.5);
                // Make sure more points are closer to the origin
                v *= 0.1 + 0.9 * scale * scale;
                kernel.push(v);
            }
            uniform.set_uniform_3fv(&kernel);
        }
        let view_uniform = program.get_uniform("View");
        let projection_uniform = program.get_uniform("Projection");
        let ambient_col_uniform = program.get_uniform("ambientLightColor");
        let light_dir_uniform = program.get_uniform("sunLightDirection");
        let light_col_uniform = program.get_uniform("sunLightColor");
        let fog_col_uniform = program.get_uniform("fogColor");
        let fog_start_uniform = program.get_uniform("fogStart");
        let fog_end_uniform = program.get_uniform("fogEnd");
        let ssao_texture_scale_uniform = program.get_uniform("ssaoTextureScale");

        let random_texture = Some(
            Texture::load(
                &assets.assets_path("textures/random.png"),
                gl,
                TextureFormat::RGBA8,
                TextureWrap::Repeat,
                TextureFilter::Linear,
            )
            .unwrap(),
        );

        // Vertex array
        let mut _vbo = ArrayBuffer::new(gl);
        // Vertex array object
        let vao = VertexArray::new(gl);
        vao.bind();
        _vbo.bind();
        let mut quad = Vec::new();
        quad.push(QuadVertex::new(-1.0, 1.0, 0.0, 1.0));
        quad.push(QuadVertex::new(-1.0, -1.0, 0.0, 0.0));
        quad.push(QuadVertex::new(1.0, 1.0, 1.0, 1.0));
        quad.push(QuadVertex::new(1.0, -1.0, 1.0, 0.0));
        _vbo.static_draw_data(&quad, false);
        QuadVertex::vertex_attrib_pointers(gl);

        // Setup buffers to render to
        let position_buffer = Texture::new_uninitialized(
            gl,
            width,
            height,
            TextureFormat::RGBA16F,
            TextureWrap::Clamp,
            TextureFilter::Nearest,
        )?;
        let color_buffer = Texture::new_uninitialized(
            gl,
            width,
            height,
            TextureFormat::RGBA16F,
            TextureWrap::None,
            TextureFilter::Nearest,
        )?;
        let normal_buffer = Texture::new_uninitialized(
            gl,
            width,
            height,
            TextureFormat::RGBA16F,
            TextureWrap::None,
            TextureFilter::Nearest,
        )?;
        let lights_buffer = Texture::new_uninitialized(
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
        let framebuffer = FrameBuffer::new(gl, width, height, true, buffers)?;

        Ok(DeferredPipeline {
            gl: gl.clone(),
            program,
            framebuffer,
            position_buffer,
            color_buffer,
            normal_buffer,
            lights_buffer,
            view_uniform,
            projection_uniform,
            ambient_col_uniform,
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

    pub fn bind(&self) {
        self.framebuffer.bind();
    }

    pub fn unbind(&self) {
        self.framebuffer.unbind();
    }

    pub fn render_to_screen(
        &mut self,
        camera: &PerspectiveCamera,
        width: u32,
        height: u32,
        ambient_col: &Vec3,
        sun_dir: &Vec3,
        sun_col: &Vec3,
        fog_col: &Vec3,
        fog_start: f32,
        fog_end: f32,
    ) {
        self.framebuffer.copy_depth_to_default();
        self.program.set_used();
        if let Some(uniform) = &self.view_uniform {
            uniform.set_uniform_matrix_4fv(camera.get_view());
        }
        if let Some(uniform) = &self.projection_uniform {
            uniform.set_uniform_matrix_4fv(camera.get_projection());
        }
        if let Some(uniform) = &self.ambient_col_uniform {
            uniform.set_uniform_3f(ambient_col);
        }
        if let Some(uniform) = &self.light_dir_uniform {
            uniform.set_uniform_3f(sun_dir);
        }
        if let Some(uniform) = &self.light_col_uniform {
            uniform.set_uniform_3f(sun_col);
        }
        if let Some(uniform) = &self.fog_col_uniform {
            uniform.set_uniform_3f(fog_col);
        }
        if let Some(uniform) = &self.fog_start_uniform {
            uniform.set_uniform_1f(fog_start);
        }
        if let Some(uniform) = &self.fog_end_uniform {
            uniform.set_uniform_1f(fog_end);
        }
        if let Some(uniform) = &self.ssao_texture_scale_uniform {
            let texture_size = 64.0;
            let texture_ratio =
                Vec2::new(width as f32 / texture_size, height as f32 / texture_size);
            uniform.set_uniform_2f(&texture_ratio);
        }
        self.vao.bind();
        unsafe {
            self.gl.Disable(gl::CULL_FACE);
            self.gl.Disable(gl::DEPTH_TEST);
            self.gl.Disable(gl::BLEND);
            self.position_buffer.bind_at(0);
            self.color_buffer.bind_at(1);
            self.normal_buffer.bind_at(2);
            self.lights_buffer.bind_at(3);
            if let Some(random_texture) = &self.random_texture {
                random_texture.bind_at(4);
            }
            self.gl.DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
    }
}
