#![allow(dead_code)]

use crate::*;
use failure::*;
use gl;
use image::GenericImageView;
use image::{self, DynamicImage};
use log::*;
use std::{
    os::raw,
    path::{Path, PathBuf},
};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "I/O error")]
    Io(#[cause] std::io::Error),
    #[fail(display = "Failed to load image {}", name)]
    FailedToLoadImage {
        name: String,
        #[cause]
        inner: image::ImageError,
    },
    #[fail(display = "Image {} is not RGBA", name)]
    ImageIsNotRgba { name: String },
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TextureFormat {
    R8,
    R16,
    RGB8,
    RGBA8,
    RGBA16F,
}

impl TextureFormat {
    pub fn internal_format(&self) -> gl::types::GLint {
        match self {
            TextureFormat::R8 => gl::R8 as gl::types::GLint,
            TextureFormat::R16 => gl::R16 as gl::types::GLint,
            TextureFormat::RGB8 => gl::RGB8 as gl::types::GLint,
            TextureFormat::RGBA8 => gl::RGBA8 as gl::types::GLint,
            TextureFormat::RGBA16F => gl::RGBA16F as gl::types::GLint,
        }
    }

    pub fn upload_format(&self) -> gl::types::GLenum {
        match self {
            TextureFormat::R8 => gl::RED,
            TextureFormat::R16 => gl::RED,
            TextureFormat::RGB8 => gl::RGB,
            TextureFormat::RGBA8 => gl::RGBA,
            TextureFormat::RGBA16F => gl::RGBA,
        }
    }

    pub fn upload_type(&self) -> gl::types::GLuint {
        match self {
            TextureFormat::R8 => gl::UNSIGNED_BYTE,
            TextureFormat::R16 => gl::UNSIGNED_SHORT,
            TextureFormat::RGB8 => gl::UNSIGNED_BYTE,
            TextureFormat::RGBA8 => gl::UNSIGNED_BYTE,
            TextureFormat::RGBA16F => gl::FLOAT,
        }
    }

    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            TextureFormat::R8 => 1,
            TextureFormat::R16 => 2,
            TextureFormat::RGB8 => 3,
            TextureFormat::RGBA8 => 4,
            TextureFormat::RGBA16F => 8,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TextureFilter {
    Linear,
    Nearest,
    MipMapLinear,
    MipMapNearest,
}

impl TextureFilter {
    pub fn min_filter(&self) -> gl::types::GLint {
        match self {
            TextureFilter::Linear => gl::LINEAR as gl::types::GLint,
            TextureFilter::Nearest => gl::NEAREST as gl::types::GLint,
            TextureFilter::MipMapLinear => gl::LINEAR_MIPMAP_LINEAR as gl::types::GLint,
            TextureFilter::MipMapNearest => gl::NEAREST_MIPMAP_NEAREST as gl::types::GLint,
        }
    }

    pub fn mag_filter(&self) -> gl::types::GLint {
        match self {
            TextureFilter::Linear => gl::LINEAR as gl::types::GLint,
            TextureFilter::Nearest => gl::NEAREST as gl::types::GLint,
            TextureFilter::MipMapLinear => gl::LINEAR as gl::types::GLint,
            TextureFilter::MipMapNearest => gl::NEAREST as gl::types::GLint,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TextureWrap {
    None,
    Repeat,
    Clamp,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TextureTarget {
    Normal,
    Array2D,
}

impl TextureTarget {
    pub fn target(&self) -> gl::types::GLuint {
        match self {
            TextureTarget::Normal => gl::TEXTURE_2D,
            TextureTarget::Array2D => gl::TEXTURE_2D_ARRAY,
        }
    }
}

pub struct Texture {
    gl: gl::Gl,
    pub handle: gl::types::GLuint,
    target: TextureTarget,
    format: TextureFormat,
    filter: TextureFilter,
    width: u32,
    height: u32,
}

impl Texture {
    /// Create a buffer texture for attachment to a framebuffer
    pub fn new_uninitialized(
        gl: &gl::Gl,
        width: u32,
        height: u32,
        format: TextureFormat,
        wrap: TextureWrap,
        filter: TextureFilter,
    ) -> Result<Texture, failure::Error> {
        let null_pointer = 0;
        let target = TextureTarget::Normal;
        let texture = Texture::start_setup(gl, width, height, target, format, wrap, filter)?;
        unsafe {
            gl.TexImage2D(
                target.target(),
                0,
                format.internal_format(),
                width as gl::types::GLsizei,
                height as gl::types::GLsizei,
                0,
                format.upload_format(),
                format.upload_type(),
                null_pointer as *const std::os::raw::c_void,
            );
        }
        texture.finish_setup();
        Ok(texture)
    }

    pub fn from_buffer(
        buffer: &Vec<u8>,
        width: u32,
        height: u32,
        format: TextureFormat,
        wrap: TextureWrap,
        filter: TextureFilter,
        gl: &gl::Gl,
    ) -> Result<Texture, failure::Error> {
        assert!(width * height * format.bytes_per_pixel() == buffer.len() as u32);
        let target = TextureTarget::Normal;
        let texture = Texture::start_setup(gl, width, height, target, format, wrap, filter)?;
        unsafe {
            gl.TexImage2D(
                target.target(),
                0,
                format.internal_format(),
                width as gl::types::GLsizei,
                height as gl::types::GLsizei,
                0,
                format.upload_format(),
                format.upload_type(),
                buffer.as_ptr() as *const raw::c_void,
            );
        }
        texture.finish_setup();
        Ok(texture)
    }

    /// Create a new empty texture
    pub fn new_empty(
        width: u32,
        height: u32,
        format: TextureFormat,
        wrap: TextureWrap,
        filter: TextureFilter,
        gl: &gl::Gl,
    ) -> Result<Texture, failure::Error> {
        let tex: Vec<u8> = vec![0; (width * height * format.bytes_per_pixel() as u32) as usize];
        let target = TextureTarget::Normal;
        let texture = Texture::start_setup(gl, width, height, target, format, wrap, filter)?;
        unsafe {
            gl.TexImage2D(
                target.target(),
                0,
                format.internal_format(),
                width as gl::types::GLsizei,
                height as gl::types::GLsizei,
                0,
                format.upload_format(),
                format.upload_type(),
                tex.as_ptr() as *const raw::c_void,
            );
        }
        texture.finish_setup();
        Ok(texture)
    }

    /// Clear and resize the texture
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let tex: Vec<u8> =
            vec![0; (width * height * self.format.bytes_per_pixel() as u32) as usize];
        self.bind();
        unsafe {
            self.gl.TexImage2D(
                self.target.target(),
                0,
                self.format.internal_format(),
                width as gl::types::GLsizei,
                height as gl::types::GLsizei,
                0,
                self.format.upload_format(),
                self.format.upload_type(),
                tex.as_ptr() as *const raw::c_void,
            );
        }
    }

    /// Load a texture from disk
    pub fn load(
        path: &Path,
        gl: &gl::Gl,
        format: TextureFormat,
        wrap: TextureWrap,
        filter: TextureFilter,
    ) -> Result<Texture, failure::Error> {
        let image = image::open(path).map_err(|e| Error::FailedToLoadImage {
            name: path.to_str().unwrap().to_string(),
            inner: e,
        })?;
        Texture::from_image(gl, &image, format, wrap, filter)
    }

    /// Load a texture from disk
    pub fn from_image(
        gl: &gl::Gl,
        image: &DynamicImage,
        format: TextureFormat,
        wrap: TextureWrap,
        filter: TextureFilter,
    ) -> Result<Texture, failure::Error> {
        let target = TextureTarget::Normal;
        let (img_width, img_height) = image.dimensions();
        let bytes = image.to_bytes();

        assert!(bytes.len() == (img_width * img_height * format.bytes_per_pixel()) as usize);

        let texture =
            Texture::start_setup(gl, img_width, img_height, target, format, wrap, filter)?;

        unsafe {
            if format.bytes_per_pixel() % 4 != 0 {
                gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            }
            gl.TexImage2D(
                target.target(),
                0,
                format.internal_format(),
                img_width as gl::types::GLsizei,
                img_height as gl::types::GLsizei,
                0,
                format.upload_format(),
                format.upload_type(),
                bytes.as_ptr() as *const raw::c_void,
            );
            if format.bytes_per_pixel() % 4 != 0 {
                gl.PixelStorei(gl::UNPACK_ALIGNMENT, 4);
            }
        }

        texture.finish_setup();
        Ok(texture)
    }

    /// Load a texture array from disk
    /// The order of layers will correspond to the order of files in the array
    pub fn load_array(
        paths: Vec<PathBuf>,
        format: TextureFormat,
        wrap: TextureWrap,
        filter: TextureFilter,
        gl: &gl::Gl,
    ) -> Result<Texture, failure::Error> {
        assert!(paths.len() > 0);
        let target = TextureTarget::Array2D;
        let mut data = Vec::new();
        let mut img_width = 0;
        let mut img_height = 0;
        let mut total_bytes = 0;
        for path in &paths {
            let face_img = image::open(path).map_err(|e| Error::FailedToLoadImage {
                name: path.to_str().unwrap().to_string(),
                inner: e,
            })?;
            let (this_img_width, this_img_height) = face_img.dimensions();
            let this_img_bytes = face_img.to_bytes();
            assert!(
                this_img_bytes.len()
                    == (this_img_width * this_img_height * format.bytes_per_pixel()) as usize
            );
            if img_width == 0 {
                img_width = this_img_width;
            } else {
                assert!(img_width == this_img_width);
            }
            if img_height == 0 {
                img_height = this_img_height;
            } else {
                assert!(img_height == this_img_height);
            }
            total_bytes += this_img_bytes.len();
            data.extend(this_img_bytes);
        }

        debug!(
            "Setup array texture with {} layers and {} bytes",
            paths.len(),
            total_bytes
        );

        let texture =
            Texture::start_setup(gl, img_width, img_height, target, format, wrap, filter)?;
        assert!(
            img_width as usize
                * img_height as usize
                * paths.len()
                * format.bytes_per_pixel() as usize
                == data.len()
        );
        unsafe {
            gl.TexImage3D(
                target.target(),
                0,
                format.internal_format(),
                img_width as gl::types::GLsizei,
                img_height as gl::types::GLsizei,
                paths.len() as gl::types::GLsizei,
                0,
                format.upload_format(),
                format.upload_type(),
                data.as_ptr() as *const raw::c_void,
            );
        }

        texture.finish_setup();
        Ok(texture)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Write pixels to the texture
    pub fn write(&self, x_offset: u32, y_offset: u32, width: u32, height: u32, pixels: &[u8]) {
        self.bind();
        if self.format.bytes_per_pixel() % 4 != 0 {
            unsafe {
                self.gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            }
        }
        unsafe {
            self.gl.TexSubImage2D(
                self.target.target(),
                0,
                x_offset as gl::types::GLint,
                y_offset as gl::types::GLint,
                width as gl::types::GLsizei,
                height as gl::types::GLsizei,
                self.format.upload_format(),
                self.format.upload_type(),
                pixels.as_ptr() as *const raw::c_void,
            );
        }
        if self.format.bytes_per_pixel() % 4 != 0 {
            unsafe {
                self.gl.PixelStorei(gl::UNPACK_ALIGNMENT, 4);
            }
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindTexture(self.target.target(), self.handle);
        }
    }

    pub fn bind_at(&self, index: u32) {
        unsafe {
            self.gl.ActiveTexture(gl::TEXTURE0 + index);
        }
        self.bind();
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.BindTexture(self.target.target(), 0);
        }
    }

    fn start_setup(
        gl: &gl::Gl,
        width: u32,
        height: u32,
        target: TextureTarget,
        format: TextureFormat,
        wrap: TextureWrap,
        filter: TextureFilter,
    ) -> Result<Texture, failure::Error> {
        let mut handle: gl::types::GLuint = 0;

        unsafe {
            gl.GenTextures(1, &mut handle);
            gl.BindTexture(target.target(), handle);
        }

        let wrap_param = match wrap {
            TextureWrap::Repeat => Some(gl::REPEAT as gl::types::GLint),
            TextureWrap::Clamp => Some(gl::CLAMP_TO_EDGE as gl::types::GLint),
            _ => None,
        };
        if let Some(param) = wrap_param {
            unsafe {
                gl.TexParameteri(target.target(), gl::TEXTURE_WRAP_S, param);
                gl.TexParameteri(target.target(), gl::TEXTURE_WRAP_T, param);
            }
        }
        unsafe {
            gl.TexParameteri(target.target(), gl::TEXTURE_MIN_FILTER, filter.min_filter());
            gl.TexParameteri(target.target(), gl::TEXTURE_MAG_FILTER, filter.mag_filter());
        }
        if filter != TextureFilter::MipMapLinear && filter != TextureFilter::MipMapNearest {
            unsafe {
                gl.TexParameteri(target.target(), gl::TEXTURE_BASE_LEVEL, 0);
                gl.TexParameteri(target.target(), gl::TEXTURE_MAX_LEVEL, 0);
            }
        }

        Ok(Texture {
            gl: gl.clone(),
            handle,
            target,
            format,
            filter,
            width,
            height,
        })
    }

    fn finish_setup(&self) {
        if self.filter == TextureFilter::MipMapLinear || self.filter == TextureFilter::MipMapNearest
        {
            unsafe {
                self.gl.GenerateMipmap(self.target.target());
            }
        }
        unsafe {
            self.gl.BindTexture(self.target.target(), 0);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { self.gl.DeleteTextures(1, &mut self.handle) };
    }
}
