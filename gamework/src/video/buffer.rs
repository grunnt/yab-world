use glow::*;

pub struct VBO {
    vbo: Buffer,
}

impl VBO {
    pub fn new(gl: &glow::Context) -> Self {
        let vbo = unsafe { gl.create_buffer().unwrap() };
        VBO { vbo }
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
        }
    }

    pub fn static_draw_data<T>(&self, gl: &glow::Context, data: &[T]) {
        unsafe {
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, to_byte_slice(data), glow::STATIC_DRAW);
        }
    }

    pub fn dynamic_draw_data<T>(&self, gl: &glow::Context, data: &[T]) {
        unsafe {
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, to_byte_slice(data), glow::DYNAMIC_DRAW);
        }
    }

    pub fn stream_draw_data<T>(&self, gl: &glow::Context, data: &[T]) {
        unsafe {
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, to_byte_slice(data), glow::STREAM_DRAW);
        }
    }

    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
    }

    /// Drop the buffer. This is not needed on shutdown, as it will be cleaned up along with the OpenGL context.
    pub fn drop(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_buffer(self.vbo);
        }
    }
}

pub struct EBO {
    ebo: Buffer,
}

impl EBO {
    pub fn new(gl: &glow::Context) -> Self {
        let ebo = unsafe { gl.create_buffer().unwrap() };
        EBO { ebo }
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
        }
    }

    pub fn static_draw_data<T>(&self, gl: &glow::Context, data: &[T]) {
        unsafe {
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                to_byte_slice(data),
                glow::STATIC_DRAW,
            );
        }
    }

    pub fn dynamic_draw_data<T>(&self, gl: &glow::Context, data: &[T]) {
        unsafe {
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                to_byte_slice(data),
                glow::DYNAMIC_DRAW,
            );
        }
    }

    pub fn stream_draw_data<T>(&self, gl: &glow::Context, data: &[T]) {
        unsafe {
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                to_byte_slice(data),
                glow::STREAM_DRAW,
            );
        }
    }

    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        }
    }

    /// Drop the buffer. This is not needed on shutdown, as it will be cleaned up along with the OpenGL context.
    pub fn drop(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_buffer(self.ebo);
        }
    }
}

pub struct VAO {
    vao: VertexArray,
}

impl VAO {
    pub fn new(gl: &glow::Context) -> Self {
        let vao = unsafe { gl.create_vertex_array().unwrap() };

        VAO { vao }
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
        }
    }

    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(None);
        }
    }

    /// Drop the buffer. This is not needed on shutdown, as it will be cleaned up along with the OpenGL context.
    pub fn drop(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.vao);
        }
    }
}

pub unsafe fn to_byte_slice<T>(slice: &[T]) -> &[u8] {
    std::slice::from_raw_parts(
        slice.as_ptr().cast(),
        slice.len() * std::mem::size_of::<T>(),
    )
}
