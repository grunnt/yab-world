pub mod buffer;
mod camera;
pub mod color;
pub mod data;
mod framebuffer;
mod gl_utils;
mod mesh;
mod primitive_render;
mod shader;
mod sprite_batcher;
mod text;
mod texture;
mod texture_array;
mod texture_atlas;
mod particles;

pub use buffer::*;
pub use camera::*;
use color::ColorRGB;
pub use framebuffer::FrameBuffer;
pub use gl;
pub use gl_utils::GlUtils;
pub use mesh::{Mesh, Vertex};
pub use primitive_render::PrimitiveRender;
pub use render_derive::VertexAttribPointers;
pub use shader::{Program, Uniform};
pub use sprite_batcher::SpriteBatcher;
pub use text::{Text, TextAlignment};
pub use texture::*;
pub use texture_array::TextureArray;
pub use texture_atlas::*;
pub use particles::*;

use crate::Size;

pub struct Video {
    gl: gl::Gl,
    ui_camera: OrthographicCamera,
    width: u32,
    height: u32,
    dpi: f32,
    background_color: ColorRGB,
}

impl Video {
    pub fn new(gl: gl::Gl, width: u32, height: u32, dpi: f32) -> Video {
        unsafe {
            gl.Viewport(0, 0, width as i32, height as i32);
        }

        // Create an orthographic camera for use in GUI
        let ui_camera = OrthographicCamera::new(width, height);

        Video {
            gl,
            ui_camera,
            width,
            height,
            dpi,
            background_color: ColorRGB::new(0.0, 0.0, 0.0),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        unsafe {
            self.gl.Viewport(0, 0, width as i32, height as i32);
        }
        self.ui_camera.set_size(self.width, self.height);
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn screen_size(&self) -> Size {
        Size::new(self.width as f32, self.height as f32)
    }

    pub fn aspect_ratio(&self) -> f32 {
        aspect_ratio(self.width, self.height)
    }

    pub fn dpi(&self) -> f32 {
        self.dpi
    }

    /// Get a reference to the OpenGl context
    pub fn gl(&self) -> &gl::Gl {
        &self.gl
    }

    /// Clear the screen
    pub fn clear_screen(&self) {
        self.gl.clear(
            self.background_color.r,
            self.background_color.g,
            self.background_color.b,
        );
    }

    pub fn ui_camera(&self) -> &OrthographicCamera {
        &self.ui_camera
    }

    pub fn set_background_color(&mut self, color: &ColorRGB) {
        self.background_color = color.clone();
    }
}

pub fn aspect_ratio(width: u32, height: u32) -> f32 {
    width as f32 / height as f32
}
