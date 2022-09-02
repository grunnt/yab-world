use crate::*;
use failure::*;
use glow::HasContext;
use glow::Program;
use glow::UniformLocation;
use nalgebra_glm::{Mat4, Vec2, Vec3, Vec4};
use std;
use std::ffi;
use std::fs;
use std::io;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "I/O error")]
    Io(#[cause] io::Error),
    #[fail(display = "Cannot read file {}", name)]
    ReadFile { name: String },
    #[fail(display = "Nul string error")]
    NulString(#[cause] ffi::NulError),
    #[fail(display = "Can not determine shader type for resource {}", name)]
    CanNotDetermineShaderTypeForResource { name: String },
    #[fail(display = "Failed to compile shader {}: {}", name, message)]
    CompileError { name: String, message: String },
    #[fail(display = "Failed to link program: {}", message)]
    LinkError { message: String },
}

impl From<ffi::NulError> for Error {
    fn from(other: ffi::NulError) -> Self {
        Error::NulString(other)
    }
}
pub struct ShaderProgram {
    program: Program,
}

impl ShaderProgram {
    pub fn from_strings(
        gl: &glow::Context,
        vertex_shader_source: &str,
        fragment_shader_source: &str,
        name: String,
    ) -> Result<Self, Error> {
        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::FRAGMENT_SHADER, fragment_shader_source),
        ];

        unsafe {
            let program = gl.create_program().unwrap();

            let mut shaders = Vec::with_capacity(shader_sources.len());

            for (shader_type, shader_source) in shader_sources.iter() {
                let shader = gl.create_shader(*shader_type).unwrap();
                gl.shader_source(shader, shader_source);
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    panic!("{}", gl.get_shader_info_log(shader));
                }
                gl.attach_shader(program, shader);
                shaders.push(shader);
            }

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("{} in shader {}", gl.get_program_info_log(program), name);
            }

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            Ok(ShaderProgram { program })
        }
    }

    pub fn load(
        gl: &glow::Context,
        assets: &Assets,
        vertex_shader_file: &str,
        fragment_shader_file: &str,
        name: String,
    ) -> Result<Self, Error> {
        let vertex_shader_source = fs::read_to_string(assets.assets_path(vertex_shader_file))
            .map_err(|_| Error::ReadFile {
                name: vertex_shader_file.to_string(),
            })?;

        let fragment_shader_source = fs::read_to_string(assets.assets_path(fragment_shader_file))
            .map_err(|_| Error::ReadFile {
            name: fragment_shader_file.to_string(),
        })?;

        ShaderProgram::from_strings(gl, &vertex_shader_source, &fragment_shader_source, name)
    }

    pub fn set_used(&self, gl: &glow::Context) {
        unsafe {
            gl.use_program(Some(self.program));
        }
    }

    pub fn get_uniform(&self, gl: &glow::Context, name: &str) -> Option<UniformLocation> {
        unsafe { gl.get_uniform_location(self.program, name) }
    }

    pub fn set_uniform_1i(&self, gl: &glow::Context, uniform: &UniformLocation, value: i32) {
        unsafe {
            gl.uniform_1_i32(Some(uniform), value);
        }
    }

    pub fn set_uniform_1f(&self, gl: &glow::Context, uniform: &UniformLocation, value: f32) {
        unsafe {
            gl.uniform_1_f32(Some(uniform), value);
        }
    }

    pub fn set_uniform_2f(&self, gl: &glow::Context, uniform: &UniformLocation, value: &Vec2) {
        unsafe {
            gl.uniform_2_f32(Some(uniform), value.x, value.y);
        }
    }

    pub fn set_uniform_3f(&self, gl: &glow::Context, uniform: &UniformLocation, value: &Vec3) {
        unsafe {
            gl.uniform_3_f32(Some(uniform), value.x, value.y, value.z);
        }
    }

    pub fn set_uniform_3fv(&self, gl: &glow::Context, uniform: &UniformLocation, value: &Vec<f32>) {
        unsafe {
            gl.uniform_3_f32_slice(Some(uniform), value);
        }
    }

    pub fn set_uniform_4f(&self, gl: &glow::Context, uniform: &UniformLocation, value: &Vec4) {
        unsafe {
            gl.uniform_4_f32(Some(uniform), value.x, value.y, value.z, value.w);
        }
    }

    pub fn set_uniform_matrix_4fv(
        &self,
        gl: &glow::Context,
        uniform: &UniformLocation,
        value: &Mat4,
    ) {
        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(uniform), false, value.as_slice());
        }
    }

    pub fn drop(&mut self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.program);
        }
    }
}
