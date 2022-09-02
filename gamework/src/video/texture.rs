#![allow(dead_code)]

use failure::*;
use glow::{HasContext, Texture};
use image::GenericImageView;
use image::{self, DynamicImage};
use log::*;
use std::path::{Path, PathBuf};

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
    SRGBA8,
    RGBA16F,
}

impl TextureFormat {
    pub fn internal_format(&self) -> i32 {
        match self {
            TextureFormat::R8 => glow::R8 as i32,
            TextureFormat::R16 => glow::R16 as i32,
            TextureFormat::RGB8 => glow::RGB8 as i32,
            TextureFormat::RGBA8 => glow::RGBA8 as i32,
            TextureFormat::SRGBA8 => glow::SRGB8_ALPHA8 as i32,
            TextureFormat::RGBA16F => glow::RGBA16F as i32,
        }
    }

    pub fn upload_format(&self) -> u32 {
        match self {
            TextureFormat::R8 => glow::RED,
            TextureFormat::R16 => glow::RED,
            TextureFormat::RGB8 => glow::RGB,
            TextureFormat::RGBA8 => glow::RGBA,
            TextureFormat::SRGBA8 => glow::RGBA,
            TextureFormat::RGBA16F => glow::RGBA,
        }
    }

    pub fn upload_type(&self) -> u32 {
        match self {
            TextureFormat::R8 => glow::UNSIGNED_BYTE,
            TextureFormat::R16 => glow::UNSIGNED_SHORT,
            TextureFormat::RGB8 => glow::UNSIGNED_BYTE,
            TextureFormat::RGBA8 => glow::UNSIGNED_BYTE,
            TextureFormat::SRGBA8 => glow::UNSIGNED_BYTE,
            TextureFormat::RGBA16F => glow::FLOAT,
        }
    }

    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            TextureFormat::R8 => 1,
            TextureFormat::R16 => 2,
            TextureFormat::RGB8 => 3,
            TextureFormat::RGBA8 => 4,
            TextureFormat::SRGBA8 => 4,
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
    pub fn min_filter(&self) -> i32 {
        match self {
            TextureFilter::Linear => glow::LINEAR as i32,
            TextureFilter::Nearest => glow::NEAREST as i32,
            TextureFilter::MipMapLinear => glow::LINEAR_MIPMAP_LINEAR as i32,
            TextureFilter::MipMapNearest => glow::NEAREST_MIPMAP_NEAREST as i32,
        }
    }

    pub fn mag_filter(&self) -> i32 {
        match self {
            TextureFilter::Linear => glow::LINEAR as i32,
            TextureFilter::Nearest => glow::NEAREST as i32,
            TextureFilter::MipMapLinear => glow::LINEAR as i32,
            TextureFilter::MipMapNearest => glow::NEAREST as i32,
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
    pub fn target(&self) -> u32 {
        match self {
            TextureTarget::Normal => glow::TEXTURE_2D,
            TextureTarget::Array2D => glow::TEXTURE_2D_ARRAY,
        }
    }
}

pub struct MyTexture {
    pub handle: Texture,
    target: TextureTarget,
    format: TextureFormat,
    filter: TextureFilter,
    width: u32,
    height: u32,
}

impl MyTexture {
    /// Create a buffer texture for attachment to a framebuffer
    pub fn new_uninitialized(
        gl: &glow::Context,
        width: u32,
        height: u32,
        format: TextureFormat,
        wrap: TextureWrap,
        filter: TextureFilter,
    ) -> Result<MyTexture, failure::Error> {
        let target = TextureTarget::Normal;
        let texture = MyTexture::start_setup(gl, width, height, target, format, wrap, filter)?;
        unsafe {
            gl.tex_image_2d(
                target.target(),
                0,
                format.internal_format(),
                width as i32,
                height as i32,
                0,
                format.upload_format(),
                format.upload_type(),
                None,
            );
        }
        texture.finish_setup(gl);
        Ok(texture)
    }

    pub fn from_buffer(
        buffer: &Vec<u8>,
        width: u32,
        height: u32,
        format: TextureFormat,
        wrap: TextureWrap,
        filter: TextureFilter,
        gl: &glow::Context,
    ) -> Result<MyTexture, failure::Error> {
        assert!(width * height * format.bytes_per_pixel() == buffer.len() as u32);
        let target = TextureTarget::Normal;
        let texture = MyTexture::start_setup(gl, width, height, target, format, wrap, filter)?;
        unsafe {
            gl.tex_image_2d(
                target.target(),
                0,
                format.internal_format(),
                width as i32,
                height as i32,
                0,
                format.upload_format(),
                format.upload_type(),
                Some(buffer),
            );
        }
        texture.finish_setup(gl);
        Ok(texture)
    }

    /// Create a new empty texture
    pub fn new_empty(
        width: u32,
        height: u32,
        format: TextureFormat,
        wrap: TextureWrap,
        filter: TextureFilter,
        gl: &glow::Context,
    ) -> Result<MyTexture, failure::Error> {
        let tex: Vec<u8> = vec![0; (width * height * format.bytes_per_pixel() as u32) as usize];
        let target = TextureTarget::Normal;
        let texture = MyTexture::start_setup(gl, width, height, target, format, wrap, filter)?;
        unsafe {
            gl.tex_image_2d(
                target.target(),
                0,
                format.internal_format(),
                width as i32,
                height as i32,
                0,
                format.upload_format(),
                format.upload_type(),
                Some(&tex),
            );
        }
        texture.finish_setup(gl);
        Ok(texture)
    }

    /// Clear and resize the texture
    pub fn resize(&mut self, gl: &glow::Context, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let tex: Vec<u8> =
            vec![0; (width * height * self.format.bytes_per_pixel() as u32) as usize];
        self.bind(gl);
        unsafe {
            gl.tex_image_2d(
                self.target.target(),
                0,
                self.format.internal_format(),
                width as i32,
                height as i32,
                0,
                self.format.upload_format(),
                self.format.upload_type(),
                Some(&tex),
            );
        }
    }

    /// Load a texture from disk
    pub fn load(
        path: &Path,
        gl: &glow::Context,
        format: TextureFormat,
        wrap: TextureWrap,
        filter: TextureFilter,
    ) -> Result<MyTexture, failure::Error> {
        let image = image::open(path).map_err(|e| Error::FailedToLoadImage {
            name: path.to_str().unwrap().to_string(),
            inner: e,
        })?;
        MyTexture::from_image(gl, &image, format, wrap, filter)
    }

    /// Load a texture from disk
    pub fn from_image(
        gl: &glow::Context,
        image: &DynamicImage,
        format: TextureFormat,
        wrap: TextureWrap,
        filter: TextureFilter,
    ) -> Result<MyTexture, failure::Error> {
        let target = TextureTarget::Normal;
        let (img_width, img_height) = image.dimensions();
        let bytes = image.to_bytes();

        assert!(bytes.len() == (img_width * img_height * format.bytes_per_pixel()) as usize);

        let texture =
            MyTexture::start_setup(gl, img_width, img_height, target, format, wrap, filter)?;

        unsafe {
            if format.bytes_per_pixel() % 4 != 0 {
                gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            }
            gl.tex_image_2d(
                target.target(),
                0,
                format.internal_format(),
                img_width as i32,
                img_height as i32,
                0,
                format.upload_format(),
                format.upload_type(),
                Some(&bytes),
            );
            if format.bytes_per_pixel() % 4 != 0 {
                gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
            }
        }

        texture.finish_setup(gl);
        Ok(texture)
    }

    /// Load a texture array from disk
    /// The order of layers will correspond to the order of files in the array
    pub fn load_array(
        paths: Vec<PathBuf>,
        format: TextureFormat,
        wrap: TextureWrap,
        filter: TextureFilter,
        gl: &glow::Context,
    ) -> Result<MyTexture, failure::Error> {
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
            MyTexture::start_setup(gl, img_width, img_height, target, format, wrap, filter)?;
        assert!(
            img_width as usize
                * img_height as usize
                * paths.len()
                * format.bytes_per_pixel() as usize
                == data.len()
        );
        unsafe {
            gl.tex_image_3d(
                target.target(),
                0,
                format.internal_format(),
                img_width as i32,
                img_height as i32,
                paths.len() as i32,
                0,
                format.upload_format(),
                format.upload_type(),
                Some(&data),
            );
        }

        texture.finish_setup(gl);
        Ok(texture)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Write pixels to the texture
    pub fn write(
        &self,
        gl: &glow::Context,
        x_offset: u32,
        y_offset: u32,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) {
        self.bind(gl);
        if self.format.bytes_per_pixel() % 4 != 0 {
            unsafe {
                gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            }
        }
        unsafe {
            if x_offset == 0 && y_offset == 0 {
                gl.tex_image_2d(
                    self.target.target(),
                    0,
                    self.format.internal_format(),
                    width as i32,
                    height as i32,
                    0,
                    self.format.upload_format(),
                    self.format.upload_type(),
                    Some(pixels),
                );
            } else {
                gl.tex_sub_image_2d(
                    self.target.target(),
                    0,
                    x_offset as i32,
                    y_offset as i32,
                    width as i32,
                    height as i32,
                    self.format.upload_format(),
                    self.format.upload_type(),
                    glow::PixelUnpackData::Slice(pixels),
                );
            }
        }
        if self.format.bytes_per_pixel() % 4 != 0 {
            unsafe {
                gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
            }
        }
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_texture(self.target.target(), Some(self.handle));
        }
    }

    pub fn bind_at(&self, gl: &glow::Context, index: u32) {
        unsafe {
            gl.active_texture(glow::TEXTURE0 + index);
        }
        self.bind(gl);
    }

    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_texture(self.target.target(), None);
        }
    }

    fn start_setup(
        gl: &glow::Context,
        width: u32,
        height: u32,
        target: TextureTarget,
        format: TextureFormat,
        wrap: TextureWrap,
        filter: TextureFilter,
    ) -> Result<MyTexture, failure::Error> {
        let handle = unsafe { gl.create_texture().unwrap() };

        unsafe {
            gl.bind_texture(target.target(), Some(handle));
        }

        let wrap_param = match wrap {
            TextureWrap::Repeat => Some(glow::REPEAT as i32),
            TextureWrap::Clamp => Some(glow::CLAMP_TO_EDGE as i32),
            _ => None,
        };
        if let Some(param) = wrap_param {
            unsafe {
                gl.tex_parameter_i32(target.target(), glow::TEXTURE_WRAP_S, param);
                gl.tex_parameter_i32(target.target(), glow::TEXTURE_WRAP_T, param);
            }
        }

        unsafe {
            gl.tex_parameter_i32(
                target.target(),
                glow::TEXTURE_MIN_FILTER,
                filter.min_filter(),
            );
            gl.tex_parameter_i32(
                target.target(),
                glow::TEXTURE_MAG_FILTER,
                filter.mag_filter(),
            );
        }

        if filter != TextureFilter::MipMapLinear && filter != TextureFilter::MipMapNearest {
            unsafe {
                gl.tex_parameter_i32(target.target(), glow::TEXTURE_BASE_LEVEL, 0);
                gl.tex_parameter_i32(target.target(), glow::TEXTURE_MAX_LEVEL, 0);
            }
        }

        Ok(MyTexture {
            handle,
            target,
            format,
            filter,
            width,
            height,
        })
    }

    fn finish_setup(&self, gl: &glow::Context) {
        if self.filter == TextureFilter::MipMapLinear || self.filter == TextureFilter::MipMapNearest
        {
            unsafe {
                gl.generate_mipmap(self.target.target());
            }
        }
        unsafe {
            gl.bind_texture(self.target.target(), None);
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe { gl.delete_texture(self.handle) };
    }
}
