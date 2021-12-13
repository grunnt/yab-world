use super::*;
use crate::video::texture::Texture;
use std::fs;
use std::{collections::HashMap, path::Path};

pub struct TextureArray {
    texture: Texture,
    name_layer_map: HashMap<String, f32>,
}

impl TextureArray {
    /// Load a directory with png files as a texture array
    pub fn load_directory(
        source_directory: &Path,
        format: TextureFormat,
        wrap: TextureWrap,
        filter: TextureFilter,
        gl: &gl::Gl,
    ) -> TextureArray {
        let paths = match fs::read_dir(source_directory) {
            Ok(paths) => paths,
            Err(e) => {
                panic!(
                    "Error loading texture files from {}: {}",
                    source_directory.to_str().unwrap(),
                    e
                );
            }
        };
        let mut image_paths = Vec::new();
        let mut name_layer_map = HashMap::new();
        let mut layer = 0;
        for path in paths {
            let path = &path.unwrap().path();
            if path.is_file() {
                //  && path.ends_with(".png")
                let name = path.file_stem().unwrap().to_str().unwrap().to_string();
                image_paths.push(path.clone());
                name_layer_map.insert(name, layer as f32);
                layer += 1;
            }
        }

        let texture = Texture::load_array(image_paths, format, wrap, filter, gl).unwrap();

        TextureArray {
            texture,
            name_layer_map,
        }
    }

    /// Get a reference to the underlying texture
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// Get a texture layer by its name (file stem)
    pub fn get_layer_by_name(&self, name: &str) -> Option<f32> {
        if let Some(layer) = self.name_layer_map.get(name) {
            Some(*layer)
        } else {
            None
        }
    }
}
