use crate::*;
use failure::*;
use gl;
use log::*;
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

pub struct Uniform {
    gl: gl::Gl,
    location: gl::types::GLint,
}

pub struct Program {
    name: String,
    gl: gl::Gl,
    id: gl::types::GLuint,
}

impl Program {
    pub fn load(
        gl: &gl::Gl,
        assets: &Assets,
        filenames: Vec<&str>,
        name: String,
    ) -> Result<Program, Error> {
        let mut shaders = Vec::new();
        for filename in filenames {
            let shader = Shader::load(gl, assets, filename)?;
            shaders.push(shader);
        }
        Program::from_shaders(&gl, &shaders, name).map_err(|message| Error::LinkError { message })
    }

    pub fn from_shaders(
        gl: &gl::Gl,
        shaders: &Vec<Shader>,
        name: String,
    ) -> Result<Program, String> {
        let program_id = unsafe { gl.CreateProgram() };

        for shader in shaders {
            unsafe {
                gl.AttachShader(program_id, shader.id());
            }
        }

        unsafe {
            gl.LinkProgram(program_id);
        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl.GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl.GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl.GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe {
                gl.DetachShader(program_id, shader.id());
            }
        }

        Ok(Program {
            name,
            gl: gl.clone(),
            id: program_id,
        })
    }

    pub fn set_used(&self) {
        unsafe {
            self.gl.UseProgram(self.id);
        }
    }

    pub fn get_uniform(&self, uniform_name: &str) -> Option<Uniform> {
        if let Some(location) = self.get_uniform_location(uniform_name) {
            Some(Uniform {
                gl: self.gl.clone(),
                location,
            })
        } else {
            None
        }
    }

    pub fn get_uniform_location(&self, uniform_name: &str) -> Option<gl::types::GLint> {
        let cname =
            ffi::CString::new(uniform_name).expect("expected uniform name to have no nul bytes");

        let location = unsafe {
            self.gl
                .GetUniformLocation(self.id, cname.as_bytes_with_nul().as_ptr() as *const i8)
        };

        if location == -1 {
            warn!(
                "missing uniform {} requested for shader {}",
                uniform_name, self.name
            );
            return None;
        }

        Some(location)
    }
}

impl Uniform {
    pub fn set_uniform_1i(&self, value: i32) {
        unsafe {
            self.gl.Uniform1i(self.location, value);
        }
    }

    pub fn set_uniform_1f(&self, value: f32) {
        unsafe {
            self.gl.Uniform1f(self.location, value);
        }
    }

    pub fn set_uniform_2f(&self, value: &Vec2) {
        unsafe {
            self.gl.Uniform2f(self.location, value.x, value.y);
        }
    }

    pub fn set_uniform_3f(&self, value: &Vec3) {
        unsafe {
            self.gl.Uniform3f(self.location, value.x, value.y, value.z);
        }
    }

    pub fn set_uniform_4f(&self, value: &Vec4) {
        unsafe {
            self.gl
                .Uniform4f(self.location, value.x, value.y, value.z, value.w);
        }
    }

    pub fn set_uniform_3fv(&self, value: &Vec<Vec3>) {
        unsafe {
            self.gl.Uniform3fv(
                self.location,
                value.len() as gl::types::GLint,
                value.as_slice().as_ptr() as *const f32,
            );
        }
    }

    pub fn set_uniform_matrix_4fv(&self, value: &Mat4) {
        unsafe {
            self.gl.UniformMatrix4fv(
                self.location,
                1,
                gl::FALSE,
                value.as_slice().as_ptr() as *const f32,
            );
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.id);
        }
    }
}

pub struct Shader {
    gl: gl::Gl,
    id: gl::types::GLuint,
}

impl Shader {
    pub fn load(gl: &gl::Gl, assets: &Assets, filename: &str) -> Result<Shader, Error> {
        const POSSIBLE_EXT: [(&str, gl::types::GLenum); 2] =
            [(".vert", gl::VERTEX_SHADER), (".frag", gl::FRAGMENT_SHADER)];

        let shader_kind = POSSIBLE_EXT
            .iter()
            .find(|&&(file_extension, _)| filename.ends_with(file_extension))
            .map(|&(_, kind)| kind)
            .ok_or_else(|| Error::CanNotDetermineShaderTypeForResource {
                name: filename.to_string(),
            })?;

        let source =
            fs::read_to_string(assets.assets_path(filename)).map_err(|_| Error::ReadFile {
                name: filename.to_string(),
            })?;

        let source = ffi::CString::new(source.into_bytes())?;
        Shader::from_source(&gl, &source, shader_kind).map_err(|message| Error::CompileError {
            name: filename.to_string(),
            message,
        })
    }

    pub fn from_source(
        gl: &gl::Gl,
        source: &ffi::CStr,
        kind: gl::types::GLenum,
    ) -> Result<Shader, String> {
        let id = shader_from_source(gl, source, kind)?;
        Ok(Shader { gl: gl.clone(), id })
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteShader(self.id);
        }
    }
}

fn shader_from_source(
    gl: &gl::Gl,
    source: &ffi::CStr,
    kind: gl::types::GLenum,
) -> Result<gl::types::GLuint, String> {
    let id = unsafe { gl.CreateShader(kind) };
    unsafe {
        gl.ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl.CompileShader(id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = create_whitespace_cstring_with_len(len as usize);

        unsafe {
            gl.GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar,
            );
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}

fn create_whitespace_cstring_with_len(len: usize) -> ffi::CString {
    // allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill it with len spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert buffer to CString
    unsafe { ffi::CString::from_vec_unchecked(buffer) }
}
