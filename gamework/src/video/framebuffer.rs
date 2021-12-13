use failure;
use gl;

pub struct FrameBuffer {
    gl: gl::Gl,
    fbo: gl::types::GLuint,
    width: u32,
    height: u32,
}

impl FrameBuffer {
    pub fn new(
        gl: &gl::Gl,
        width: u32,
        height: u32,
        depth_attachment: bool,
        buffer_handles: Vec<gl::types::GLuint>,
    ) -> Result<FrameBuffer, failure::Error> {
        let mut fbo: gl::types::GLuint = 0;
        unsafe {
            // Create frame buffer object
            gl.GenFramebuffers(1, &mut fbo);
            gl.BindFramebuffer(gl::FRAMEBUFFER, fbo);
        }

        // Attach texture buffers to render to
        let mut buffers = Vec::new();
        let mut count = 0;
        for buffer_handle in buffer_handles {
            let attachment = gl::COLOR_ATTACHMENT0 + count;
            unsafe {
                gl.FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    attachment,
                    gl::TEXTURE_2D,
                    buffer_handle,
                    0,
                );
            }
            buffers.push(attachment);
            count = count + 1;
        }
        unsafe {
            gl.DrawBuffers(buffers.len() as gl::types::GLsizei, buffers.as_ptr());
        }

        // Create and attach depth buffer
        if depth_attachment {
            let mut rbo_depth: gl::types::GLuint = 0;
            unsafe {
                gl.GenRenderbuffers(1, &mut rbo_depth);
                gl.BindRenderbuffer(gl::RENDERBUFFER, rbo_depth);
                gl.RenderbufferStorage(
                    gl::RENDERBUFFER,
                    gl::DEPTH_COMPONENT,
                    width as i32,
                    height as i32,
                );
                gl.FramebufferRenderbuffer(
                    gl::FRAMEBUFFER,
                    gl::DEPTH_ATTACHMENT,
                    gl::RENDERBUFFER,
                    rbo_depth,
                );
            }
        }

        unsafe {
            // Check frame buffer for completeness
            if gl.CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("framebuffer is incomplete");
            }
            // Bind to default framebuffer
            gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Ok(FrameBuffer {
            gl: gl.clone(),
            fbo,
            width,
            height,
        })
    }

    // Copy the depth buffer of this framebuffer to the default framebuffer
    // This allows forward rendering to use the depth values
    pub fn copy_depth_to_default(&self) {
        unsafe {
            self.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, self.fbo);
            self.gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
            self.gl.BlitFramebuffer(
                0,
                0,
                self.width as i32,
                self.height as i32,
                0,
                0,
                self.width as i32,
                self.height as i32,
                gl::DEPTH_BUFFER_BIT,
                gl::NEAREST,
            );
        }
        self.unbind();
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteFramebuffers(1, &mut self.fbo);
        }
    }
}
