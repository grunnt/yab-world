use std::ffi::CStr;

use crate::gl::{self, Gl};

pub trait GlUtils {
    fn get_version(&self) -> String;

    fn clear(&self, r: f32, g: f32, b: f32);
}

impl GlUtils for Gl {
    /// Get OpenGL version string
    fn get_version(&self) -> String {
        unsafe {
            let data = CStr::from_ptr(self.GetString(gl::VERSION) as *const _)
                .to_bytes()
                .to_vec();
            String::from_utf8(data).unwrap()
        }
    }

    /// Clear the color and depth buffers
    fn clear(&self, r: f32, g: f32, b: f32) {
        unsafe {
            self.ClearColor(r, g, b, 1.0);
            self.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
}
