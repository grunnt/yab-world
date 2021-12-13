use super::*;
use crate::video::texture::Texture;
use image::{DynamicImage, GenericImageView, GrayImage};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    path::Path,
};
use texture_packer::{
    exporter::ImageExporter, importer::ImageImporter, TexturePacker, TexturePackerConfig,
};

pub struct TextureAtlas {
    texture: Texture,
    frames: Vec<TextureFrame>,
    name_id_map: HashMap<String, usize>,
}

impl TextureAtlas {
    /// Load a directory as a texture atlas
    pub fn load_directory(source_directory: &Path, gl: &gl::Gl) -> TextureAtlas {
        // Pack the directory
        let packer = pack_directory(source_directory);

        // Store in a texture
        let image = ImageExporter::export(&packer).unwrap();
        let texture = Texture::from_image(
            gl,
            &image,
            TextureFormat::RGBA8,
            TextureWrap::Repeat,
            TextureFilter::Linear,
        )
        .unwrap();

        let mut frames = Vec::new();
        let mut name_id_map = HashMap::new();
        let width = image.width() as f32;
        let height = image.height() as f32;
        for (name, frame) in packer.get_frames() {
            let id = frames.len();
            name_id_map.insert(name.clone(), id);
            frames.push(TextureFrame {
                x: (frame.frame.x as f32 + 0.5) / width,
                y: (frame.frame.y as f32 + 0.5) / height,
                width: frame.frame.w as f32 / width,
                height: frame.frame.h as f32 / height,
            });
        }

        TextureAtlas {
            texture,
            frames,
            name_id_map,
        }
    }

    /// Load a texture atlas
    pub fn load(png_path: &Path, json_path: &Path, gl: &gl::Gl) -> TextureAtlas {
        // Load atlas texture
        let texture = Texture::load(
            png_path,
            gl,
            TextureFormat::RGBA8,
            TextureWrap::Clamp,
            TextureFilter::Nearest,
        )
        .unwrap();

        // Load frame definition json
        let wrapped_frames = load_wrapped_frames(json_path);
        let mut frames = Vec::new();
        let mut name_id_map = HashMap::new();
        for wrapped in &wrapped_frames {
            frames.push(wrapped.frame.clone());
            name_id_map.insert(wrapped.name.clone(), wrapped.frame_id);
        }

        TextureAtlas {
            texture,
            frames,
            name_id_map,
        }
    }

    pub fn from_array(
        textures: Vec<(String, u32, u32, Vec<u8>)>,
        format: TextureFormat,
        gl: &gl::Gl,
    ) -> TextureAtlas {
        let config = TexturePackerConfig {
            max_width: 1024,
            max_height: 1024,
            allow_rotation: false,
            texture_outlines: false,
            border_padding: 2,
            texture_padding: 2,
            ..Default::default()
        };

        let mut packer = TexturePacker::new_skyline(config);

        for texture in textures {
            let image = DynamicImage::ImageLuma8(
                GrayImage::from_raw(texture.1, texture.2, texture.3).unwrap(),
            );
            packer.pack_own(texture.0, image).unwrap();
        }

        // Some trickery to turn the result into single channel
        let image = ImageExporter::export(&packer).unwrap();
        let image = DynamicImage::ImageLuma8(image.to_luma());

        let texture = Texture::from_image(
            gl,
            &image,
            format,
            TextureWrap::Clamp,
            TextureFilter::Linear,
        )
        .unwrap();

        let mut frames = Vec::new();
        let mut name_id_map = HashMap::new();
        let width = image.width() as f32;
        let height = image.height() as f32;
        for (name, frame) in packer.get_frames() {
            let id = frames.len();
            name_id_map.insert(name.clone(), id);
            frames.push(TextureFrame {
                x: frame.frame.x as f32 / width,
                y: frame.frame.y as f32 / height,
                width: frame.frame.w as f32 / width,
                height: frame.frame.h as f32 / height,
            });
        }

        TextureAtlas {
            texture,
            frames,
            name_id_map,
        }
    }

    /// Get a reference to the underlying texture of this atlas
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// Get a sub-texture frame by it's ID
    pub fn frame(&self, id: usize) -> &TextureFrame {
        self.frames.get(id).unwrap()
    }

    /// Get the size of a frame in pixels (which is stored in texture coordinates [0.0..1.0])
    pub fn calculate_frame_size_pixels(&self, frame_id: usize) -> Size {
        let frame = self.frame(frame_id);
        let (width, height) = (self.texture.width() as f32, self.texture.height() as f32);
        Size::new(frame.width * width, frame.height * height)
    }

    /// Get a sub-texture's ID by it's name
    pub fn find_id(&self, name: &str) -> Option<usize> {
        if let Some(id) = self.name_id_map.get(name) {
            Some(*id)
        } else {
            None
        }
    }

    /// Get the list of texture frames
    pub fn frames(&self) -> &Vec<TextureFrame> {
        &self.frames
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrappedTextureFrame {
    pub frame_id: usize,
    pub name: String,
    pub frame: TextureFrame,
}

pub fn load_wrapped_frames(json_path: &Path) -> Vec<WrappedTextureFrame> {
    let wrapped_frames: Vec<WrappedTextureFrame> = match fs::read_to_string(json_path) {
        Ok(json_string) => match serde_json::from_str(&json_string) {
            Ok(frames) => frames,
            Err(e) => {
                panic!("Error parsing atlas file: {}", e);
            }
        },
        Err(e) => {
            panic!("Error loading atlas file: {}", e);
        }
    };
    wrapped_frames
}

pub fn generate_texture_atlas(source_directory: &Path, target_png: &Path, target_json: &Path) {
    // Pack all textures in the directory in a single atlas
    let packer = pack_directory(source_directory);

    // Write PNG file
    let exporter = ImageExporter::export(&packer).unwrap();
    let mut file = File::create(target_png).unwrap();
    exporter
        .write_to(&mut file, image::ImageFormat::Png)
        .unwrap();

    // Write JSON file containing frame information
    let width = exporter.width() as f32;
    let height = exporter.height() as f32;
    let mut frames = Vec::new();
    for (name, frame) in packer.get_frames() {
        let frame_id = frames.len();
        frames.push(WrappedTextureFrame {
            frame_id,
            name: name.clone(),
            frame: TextureFrame {
                x: (frame.frame.x as f32 + 0.5) / width,
                y: (frame.frame.y as f32 + 0.5) / height,
                width: frame.frame.w as f32 / width,
                height: frame.frame.h as f32 / height,
            },
        });
    }
    let json_string = serde_json::to_string_pretty(&frames).unwrap();
    fs::write(target_json, json_string).unwrap();
}

fn pack_directory(source_directory: &Path) -> TexturePacker<DynamicImage> {
    let config = TexturePackerConfig {
        max_width: 1024,
        max_height: 1024,
        allow_rotation: false,
        texture_outlines: false,
        border_padding: 2,
        texture_padding: 2,
        ..Default::default()
    };

    let mut packer = TexturePacker::new_skyline(config);

    let paths = fs::read_dir(source_directory).unwrap();
    for path in paths {
        let path = &path.unwrap().path();
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let texture = ImageImporter::import_from_file(&path).unwrap();
        packer.pack_own(name, texture).unwrap();
    }

    packer
}
