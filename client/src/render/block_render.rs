use crate::render::*;
use common::{block::BlockRegistry, chunk::*};
use failure;
use gamework::video::*;
use gamework::*;
use gl;
use log::*;
use nalgebra_glm::*;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

pub struct BlockRenderer {
    program: Program,
    model_uniform: Option<Uniform>,
    view_uniform: Option<Uniform>,
    projection_uniform: Option<Uniform>,
    z_offset_uniform: Option<Uniform>,
    t_program: Program,
    t_model_uniform: Option<Uniform>,
    t_view_uniform: Option<Uniform>,
    t_projection_uniform: Option<Uniform>,
    t_alpha_uniform: Option<Uniform>,
    t_z_offset_uniform: Option<Uniform>,
    ambient_col_uniform: Option<Uniform>,
    light_dir_uniform: Option<Uniform>,
    light_col_uniform: Option<Uniform>,
    fog_col_uniform: Option<Uniform>,
    fog_start_uniform: Option<Uniform>,
    fog_end_uniform: Option<Uniform>,
    block_textures: Texture,
    pub meshes: HashMap<ChunkPos, BlockMesh>,
    pub translucent_meshes: HashMap<ChunkPos, BlockMesh>,
    pub mesh_count: usize,
    pub triangle_count: usize,
}

impl BlockRenderer {
    pub fn new(
        gl: &gl::Gl,
        assets: &Assets,
        block_registry: &BlockRegistry,
    ) -> Result<BlockRenderer, failure::Error> {
        // Normal rendering
        let program = Program::load(gl, assets, vec!["shaders/block.vert", "shaders/block.frag"])?;
        program.set_used();
        let model_uniform = program.get_uniform("Model");
        let view_uniform = program.get_uniform("View");
        let projection_uniform = program.get_uniform("Projection");
        let z_offset_uniform = program.get_uniform("zOffset");
        if let Some(uniform) = program.get_uniform("blockTextures") {
            uniform.set_uniform_1i(0);
        }
        // Build a list of texture file names sorted by texture array index
        let texture_paths = block_registry.texture_paths_sorted(assets);
        debug!("Loading {} block texture files..", texture_paths.len());

        let block_textures = Texture::load_array(
            texture_paths,
            TextureFormat::RGBA8,
            TextureWrap::None,
            TextureFilter::MipMapNearest,
            gl,
        )
        .unwrap();

        // Translucent rendering
        let t_program = Program::load(
            gl,
            assets,
            vec!["shaders/translucent.vert", "shaders/translucent.frag"],
        )?;
        t_program.set_used();
        let t_model_uniform = t_program.get_uniform("Model");
        let t_view_uniform = t_program.get_uniform("View");
        let t_projection_uniform = t_program.get_uniform("Projection");
        let t_alpha_uniform = t_program.get_uniform("Alpha");
        let t_z_offset_uniform = t_program.get_uniform("zOffset");
        let ambient_col_uniform = t_program.get_uniform("ambientLightColor");
        let light_dir_uniform = t_program.get_uniform("sunLightDirection");
        let light_col_uniform = t_program.get_uniform("sunLightColor");
        let fog_col_uniform = t_program.get_uniform("fogColor");
        let fog_start_uniform = t_program.get_uniform("fogStart");
        let fog_end_uniform = t_program.get_uniform("fogEnd");
        if let Some(uniform) = program.get_uniform("blockTextures") {
            uniform.set_uniform_1i(0);
        }

        let meshes = HashMap::new();
        let translucent_meshes = HashMap::new();
        Ok(BlockRenderer {
            meshes,
            translucent_meshes,
            program,
            model_uniform,
            view_uniform,
            projection_uniform,
            z_offset_uniform,
            block_textures,
            t_program,
            t_model_uniform,
            t_view_uniform,
            t_projection_uniform,
            t_alpha_uniform,
            t_z_offset_uniform,
            ambient_col_uniform,
            light_dir_uniform,
            light_col_uniform,
            fog_col_uniform,
            fog_start_uniform,
            fog_end_uniform,
            mesh_count: 0,
            triangle_count: 0,
        })
    }

    pub fn insert_mesh_pos(&mut self, chunk_pos: ChunkPos, mesh: BlockMesh) {
        self.meshes.insert(chunk_pos, mesh);
    }

    pub fn insert_translucent_mesh_pos(
        &mut self,
        chunk_pos: ChunkPos,
        translucent_mesh: BlockMesh,
    ) {
        self.translucent_meshes.insert(chunk_pos, translucent_mesh);
    }

    pub fn remove_mesh_pos(&mut self, chunk_pos: ChunkPos) {
        self.meshes.remove(&chunk_pos);
    }

    pub fn remove_translucent_mesh_pos(&mut self, chunk_pos: ChunkPos) {
        self.translucent_meshes.remove(&chunk_pos);
    }

    pub fn remove_col_set(&mut self, cols: &HashSet<ChunkColumnPos>) {
        self.meshes
            .retain(|p, _| !cols.contains(&ChunkColumnPos::new(p.x, p.y)));
        self.translucent_meshes
            .retain(|p, _| !cols.contains(&ChunkColumnPos::new(p.x, p.y)));
    }

    pub fn render(
        &mut self,
        gl: &gl::Gl,
        model: Mat4,
        camera: &PerspectiveCamera,
        max_range: i16,
        center_col: &ChunkColumnPos,
        render_lines: bool,
        out_of_range: &mut HashSet<ChunkColumnPos>,
    ) {
        self.program.set_used();
        if let Some(uniform) = &self.model_uniform {
            uniform.set_uniform_matrix_4fv(&model);
        }
        if let Some(uniform) = &self.view_uniform {
            uniform.set_uniform_matrix_4fv(camera.get_view());
        }
        if let Some(uniform) = &self.projection_uniform {
            uniform.set_uniform_matrix_4fv(camera.get_projection());
        }
        self.block_textures.bind_at(0);
        unsafe {
            gl.Enable(gl::CULL_FACE);
            gl.Enable(gl::DEPTH_TEST);
            gl.Disable(gl::BLEND);
        }
        let mut vertex_count = 0;
        let mut mesh_count = 0;
        let render_instant = Instant::now();
        let dst_sq = max_range * max_range;
        for (cp, mesh) in &mut self.meshes {
            let col = ChunkColumnPos::from_chunk_pos(*cp);
            if col.dist_squared_from(center_col) > dst_sq {
                out_of_range.insert(col);
            }

            if render_lines {
                unsafe {
                    gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
                }
            }

            let duration = render_instant - mesh.creation_instant;
            let duration_s =
                duration.as_secs() as f32 + duration.subsec_nanos() as f32 / 1_000_000_000.0;
            if mesh.animate && duration_s < 1.0 {
                if let Some(uniform) = &self.z_offset_uniform {
                    uniform.set_uniform_1f(0.5 + duration_s.powf(0.5) * 0.5);
                }
            }
            mesh.render(gl);
            if mesh.animate && duration_s < 1.0 {
                if let Some(uniform) = &self.z_offset_uniform {
                    uniform.set_uniform_1f(1.0);
                }
            }
            if render_lines {
                unsafe {
                    gl.PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                }
            }
            vertex_count = vertex_count + mesh.vertex_count;
            mesh_count = mesh_count + 1;
        }
        self.triangle_count = vertex_count as usize / 3;
        self.mesh_count = mesh_count;
    }

    pub fn render_translucent(
        &mut self,
        gl: &gl::Gl,
        model: Mat4,
        camera: &PerspectiveCamera,
        ambient_col: &Vec3,
        sun_dir: &Vec3,
        sun_col: &Vec3,
        fog_col: &Vec3,
        fog_start: f32,
        fog_end: f32,
    ) {
        self.t_program.set_used();
        if let Some(uniform) = &self.t_model_uniform {
            uniform.set_uniform_matrix_4fv(&model);
        }
        if let Some(uniform) = &self.t_view_uniform {
            uniform.set_uniform_matrix_4fv(camera.get_view());
        }
        if let Some(uniform) = &self.t_projection_uniform {
            uniform.set_uniform_matrix_4fv(camera.get_projection());
        }
        if let Some(uniform) = &self.t_alpha_uniform {
            uniform.set_uniform_1f(0.85);
        }
        if let Some(uniform) = &self.ambient_col_uniform {
            uniform.set_uniform_3f(ambient_col);
        }
        if let Some(uniform) = &self.light_dir_uniform {
            uniform.set_uniform_3f(sun_dir);
        }
        if let Some(uniform) = &self.light_col_uniform {
            uniform.set_uniform_3f(sun_col);
        }
        if let Some(uniform) = &self.fog_col_uniform {
            uniform.set_uniform_3f(fog_col);
        }
        if let Some(uniform) = &self.fog_start_uniform {
            uniform.set_uniform_1f(fog_start);
        }
        if let Some(uniform) = &self.fog_end_uniform {
            uniform.set_uniform_1f(fog_end);
        }
        self.block_textures.bind_at(0);
        unsafe {
            gl.Disable(gl::CULL_FACE);
            gl.Enable(gl::DEPTH_TEST);
            gl.Enable(gl::BLEND);
        }
        let mut vertex_count = self.triangle_count * 3;
        let mut mesh_count = self.mesh_count;
        let render_instant = Instant::now();
        for (_, mesh) in &mut self.translucent_meshes {
            let duration = render_instant - mesh.creation_instant;
            let duration_s =
                duration.as_secs() as f32 + duration.subsec_nanos() as f32 / 1_000_000_000.0;
            if mesh.animate && duration_s < 1.0 {
                if let Some(uniform) = &self.t_z_offset_uniform {
                    uniform.set_uniform_1f(0.5 + duration_s.powf(0.5) * 0.5);
                }
            }
            mesh.render(gl);
            if mesh.animate && duration_s < 1.0 {
                if let Some(uniform) = &self.t_z_offset_uniform {
                    uniform.set_uniform_1f(1.0);
                }
            }
            vertex_count = vertex_count + mesh.vertex_count as usize;
            mesh_count = mesh_count + 1;
        }
        self.triangle_count = vertex_count as usize / 3;
        self.mesh_count = mesh_count;
    }

    pub fn get_stats(&self) -> (usize, usize) {
        (self.mesh_count, self.triangle_count)
    }
}
