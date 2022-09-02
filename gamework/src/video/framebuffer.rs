use failure;
use glow::*;

pub struct FBO {
    fbo: Framebuffer,
    width: u32,
    height: u32,
}

impl FBO {
    pub fn new(
        gl: &glow::Context,
        width: u32,
        height: u32,
        depth_attachment: bool,
        buffer_handles: Vec<Texture>,
    ) -> Result<FBO, failure::Error> {
        let fbo = unsafe { gl.create_framebuffer().unwrap() };
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
        }

        // Attach texture buffers to render to
        let mut buffers = Vec::new();
        let mut count = 0;
        for buffer_handle in buffer_handles {
            let attachment = glow::COLOR_ATTACHMENT0 + count;
            unsafe {
                gl.framebuffer_texture_2d(
                    glow::FRAMEBUFFER,
                    attachment,
                    glow::TEXTURE_2D,
                    Some(buffer_handle),
                    0,
                );
            }
            buffers.push(attachment);
            count = count + 1;
        }
        unsafe {
            gl.draw_buffers(&buffers);
        }

        // Create and attach depth buffer
        if depth_attachment {
            unsafe {
                let rbo_depth = gl.create_renderbuffer().unwrap();
                gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rbo_depth));
                gl.renderbuffer_storage(
                    glow::RENDERBUFFER,
                    glow::DEPTH24_STENCIL8,
                    width as i32,
                    height as i32,
                );
                gl.framebuffer_renderbuffer(
                    glow::FRAMEBUFFER,
                    glow::DEPTH_ATTACHMENT,
                    glow::RENDERBUFFER,
                    Some(rbo_depth),
                );
            }
        }

        unsafe {
            // Check frame buffer for completeness
            if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
                panic!("framebuffer is incomplete");
            }
            // Bind to default framebuffer
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }

        Ok(FBO { fbo, width, height })
    }

    // Copy the depth buffer of this framebuffer to the default framebuffer
    // This allows forward rendering to use the depth values
    pub fn copy_depth_to_default(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(self.fbo));
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);
            gl.blit_framebuffer(
                0,
                0,
                self.width as i32,
                self.height as i32,
                0,
                0,
                self.width as i32,
                self.height as i32,
                glow::DEPTH_BUFFER_BIT,
                glow::NEAREST,
            );
        }
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
        }
    }

    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }

    pub fn drop(&mut self, gl: &glow::Context) {
        unsafe {
            gl.delete_framebuffer(self.fbo);
        }
    }
}
