use crate::{
    gl,
    video::{buffer, Program, TextureAtlas, Uniform},
    Size,
};
use crate::{
    video::{data, VertexAttribPointers},
    Assets,
};
use nalgebra_glm::{Mat4, Vec3};
use std::io::Read;
use std::{collections::HashMap, fs};

const SUPPORTED_CHARACTERS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789?!.,;:\"'-+*&@%<>()[]{}";

#[derive(Copy, Clone, Debug)]
pub enum TextAlignment {
    Start,
    Center,
    End,
}

pub struct Text {
    gl: gl::Gl,
    program: Program,
    color_uniform: Uniform,
    projection_uniform: Uniform,
    vbo: buffer::ArrayBuffer,
    vbo_capacity: usize,
    vao: buffer::VertexArray,
    vertices: Vec<TextVertex>,
    texture: TextureAtlas,
    fonts: Vec<TextFont>,
}

impl Text {
    pub fn new(assets: &Assets, gl: &gl::Gl, font_defs: Vec<(&str, f32)>) -> Self {
        // Setup rendering pipeline
        let program = Program::load(
            gl,
            assets,
            vec!["shaders/text.vert", "shaders/text.frag"],
            "text".to_string(),
        )
        .unwrap();
        program.set_used();
        let color_uniform = program.get_uniform("color").unwrap();
        let projection_uniform = program.get_uniform("projection").unwrap();
        program
            .get_uniform("fontTexture")
            .unwrap()
            .set_uniform_1i(0);

        // Vertex array object
        let vbo = buffer::ArrayBuffer::new(gl);
        let vao = buffer::VertexArray::new(gl);
        vao.bind();
        vbo.bind();
        TextVertex::vertex_attrib_pointers(gl);
        // Reserve space in the vbo
        let vbo_capacity = 512;
        vbo.stream_draw_data_null::<TextVertex>(vbo_capacity);

        // Load the fonts
        let mut character_textures: Vec<(String, u32, u32, Vec<u8>)> = Vec::new();
        let mut font_index = 0;
        let mut fonts = Vec::new();
        for font_def in font_defs {
            // Load font file bytes
            let font_path = assets.assets_path(font_def.0);
            let mut font_file = fs::File::open(&font_path).expect("Fond file not found");
            let font_file_metadata =
                fs::metadata(&font_path).expect("Could not read font file metadata");
            let mut font_bytes = vec![0; font_file_metadata.len() as usize];
            font_file
                .read(&mut font_bytes)
                .expect("Buffer overflow when reading font file");

            // Initialize fontdue
            let settings = fontdue::FontSettings {
                scale: font_def.1,
                ..fontdue::FontSettings::default()
            };
            let font = fontdue::Font::from_bytes(font_bytes, settings).unwrap();

            // Pre-rasterize font texture
            let mut characters = HashMap::new();
            let mut max_height = 0.0;
            let mut base_line = 0.0;
            let mut space_width = 0.0;
            let mut count = 0.0;
            for character in SUPPORTED_CHARACTERS.chars() {
                let (metrics, bytes) = font.rasterize(character, settings.scale);
                if metrics.width == 0 || metrics.height == 0 {
                    continue;
                }
                count += 1.0;
                let height = metrics.height as f32;
                if height > max_height {
                    max_height = height;
                }
                let width = metrics.width as f32;
                space_width += width;
                if (metrics.ymin as f32) < base_line {
                    base_line = metrics.ymin as f32;
                }
                characters.insert(
                    character,
                    Character {
                        character,
                        width,
                        height,
                        min_y: metrics.ymin as f32,
                        texture_frame_id: 0,
                    },
                );
                character_textures.push((
                    texture_name(font_index, character),
                    metrics.width as u32,
                    metrics.height as u32,
                    bytes,
                ));
            }
            if count > 0.0 {
                space_width /= count;
            }

            fonts.push(TextFont {
                characters,
                line_height: max_height * 1.1,
                base_line: -base_line,
                space_width: space_width * 0.5,
            });

            font_index += 1;
        }
        // Pack the font texture atlas
        let texture =
            TextureAtlas::from_array(character_textures, crate::video::TextureFormat::R8, gl);

        // Update texture frame ID's after packing
        for font_id in 0..font_index {
            for character in SUPPORTED_CHARACTERS.chars() {
                let texture_name = texture_name(font_id, character);
                if let Some(frame_id) = texture.find_id(&texture_name) {
                    fonts
                        .get_mut(font_id as usize)
                        .unwrap()
                        .characters
                        .get_mut(&character)
                        .unwrap()
                        .texture_frame_id = frame_id;
                } else {
                    panic!("texture frame for character {} not found", character);
                }
            }
        }

        Text {
            gl: gl.clone(),
            program,
            color_uniform,
            projection_uniform,
            vbo,
            vbo_capacity,
            vao,
            vertices: Vec::new(),
            texture,
            fonts,
        }
    }

    pub fn measure_string(&self, text: &str, font_index: usize) -> Size {
        let mut line_w = 0.0;
        let mut w = 0.0;
        let mut h = self.fonts[font_index].line_height;
        for character in text.chars() {
            let supported_char = self.fonts[font_index].characters.contains_key(&character);
            let space = character == ' ';
            let newline = character == '\n';
            if newline {
                if line_w > w {
                    w = line_w;
                }
                line_w = 0.0;
                h += self.fonts[font_index].line_height;
            } else if space {
                w += self.fonts[font_index].space_width;
            } else if supported_char {
                let character_frame = self.fonts[font_index]
                    .characters
                    .get(&character)
                    .unwrap()
                    .clone();
                w += character_frame.width;
            }
        }
        if line_w > w {
            w = line_w;
        }
        Size::new(w, h)
    }

    pub fn place_string(
        &mut self,
        x: f32,
        y: f32,
        text: &str,
        x_alignment: TextAlignment,
        y_alignment: TextAlignment,
        font_index: usize,
    ) {
        let text_size = self.measure_string(text, font_index);
        let mut tx = x + match x_alignment {
            TextAlignment::Start => 0.0,
            TextAlignment::Center => -text_size.width / 2.0,
            TextAlignment::End => -text_size.width,
        };
        let mut ty = y + match y_alignment {
            TextAlignment::Start => 0.0,
            TextAlignment::Center => -text_size.height / 2.0,
            TextAlignment::End => -text_size.height,
        };
        for character in text.chars() {
            let supported_char = self.fonts[font_index].characters.contains_key(&character);
            let space = character == ' ';
            let newline = character == '\n';
            if newline {
                tx = x;
                ty += self.fonts[font_index].line_height;
            } else if space {
                tx += self.fonts[font_index].space_width;
            } else if supported_char {
                let character_frame = self.fonts[font_index]
                    .characters
                    .get(&character)
                    .unwrap()
                    .clone();
                self.place_character(tx, ty, character, font_index);
                tx += character_frame.width;
            }
        }
    }

    pub fn place_character(&mut self, x: f32, y: f32, character: char, font_index: usize) {
        if !self.fonts[font_index].characters.contains_key(&character) {
            return;
        }
        let character_frame = self.fonts[font_index].characters.get(&character).unwrap();
        let frame = self.texture.frame(character_frame.texture_frame_id);
        let w = character_frame.width;
        let h = character_frame.height;
        let y = y + (self.fonts[font_index].line_height - h)
            - self.fonts[font_index].base_line
            - character_frame.min_y;

        // Triangle 1
        self.vertices.push(vertex(x, y, frame.x, frame.y));
        self.vertices
            .push(vertex(x, y + h, frame.x, frame.y + frame.height));
        self.vertices
            .push(vertex(x + w, y, frame.x + frame.width, frame.y));
        // Triangle 2
        self.vertices
            .push(vertex(x, y + h, frame.x, frame.y + frame.height));
        self.vertices.push(vertex(
            x + w,
            y + h,
            frame.x + frame.width,
            frame.y + frame.height,
        ));
        self.vertices
            .push(vertex(x + w, y, frame.x + frame.width, frame.y));
    }

    pub fn draw(&mut self, projection: &Mat4) {
        self.vbo.bind();
        // Increase buffer size if needed
        if self.vertices.len() > self.vbo_capacity {
            self.vbo_capacity = self.vertices.len() * 2;
            self.vbo
                .stream_draw_data_null::<TextVertex>(self.vbo_capacity);
        }
        // Upload the vertices
        unsafe {
            if let Some(mut buffer) = self
                .vbo
                .map_buffer_range_write_invalidate::<TextVertex>(0, self.vertices.len())
            {
                for i in 0..self.vertices.len() {
                    *buffer.get_unchecked_mut(i) = self.vertices.get(i).unwrap().clone();
                }
            }
        }
        self.vbo.unbind();

        // Now render the text
        self.vao.bind();
        unsafe {
            self.gl.Enable(gl::CULL_FACE);
            self.gl.Disable(gl::DEPTH_TEST);
            self.gl.Enable(gl::BLEND);
            self.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
        self.program.set_used();
        self.color_uniform.set_uniform_3f(&Vec3::new(1.0, 1.0, 1.0));
        self.projection_uniform.set_uniform_matrix_4fv(projection);
        self.texture.texture().bind_at(0);
        unsafe {
            self.gl
                .DrawArrays(gl::TRIANGLES, 0, self.vertices.len() as gl::types::GLsizei);
        }

        // Clear the vertex buffer for the next frame
        self.vertices.clear();
    }
}

struct TextFont {
    pub characters: HashMap<char, Character>,
    pub line_height: f32,
    pub base_line: f32,
    pub space_width: f32,
}

#[derive(Clone, Debug)]
pub struct Character {
    character: char,
    width: f32,
    height: f32,
    min_y: f32,
    texture_frame_id: usize,
}

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct TextVertex {
    #[location = "0"]
    pub position: data::f32_f32,
    #[location = "1"]
    pub tex_coords: data::f32_f32,
}

fn vertex(x: f32, y: f32, u: f32, v: f32) -> TextVertex {
    TextVertex {
        position: (x, y).into(),
        tex_coords: (u, v).into(),
    }
}

fn texture_name(font_id: usize, character: char) -> String {
    format!("{}-{}", font_id, character)
}
