use crate::render::*;
use common::{block::BlockRegistry, chunk::*};
use failure;
use gamework::glow::*;
use gamework::video::*;
use gamework::*;
use log::*;
use nalgebra_glm::*;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

pub struct BlockRenderer {
    program: ShaderProgram,
    model_uniform: Option<UniformLocation>,
    view_uniform: Option<UniformLocation>,
    projection_uniform: Option<UniformLocation>,
    z_offset_uniform: Option<UniformLocation>,
    t_program: ShaderProgram,
    t_model_uniform: Option<UniformLocation>,
    t_view_uniform: Option<UniformLocation>,
    t_projection_uniform: Option<UniformLocation>,
    t_z_offset_uniform: Option<UniformLocation>,
    light_dir_uniform: Option<UniformLocation>,
    light_col_uniform: Option<UniformLocation>,
    fog_col_uniform: Option<UniformLocation>,
    fog_start_uniform: Option<UniformLocation>,
    fog_end_uniform: Option<UniformLocation>,
    block_textures: MyTexture,
    pub meshes: HashMap<ChunkPos, BlockMesh>,
    pub translucent_meshes: HashMap<ChunkPos, BlockMesh>,
    pub mesh_count: usize,
    pub triangle_count: usize,
}

impl BlockRenderer {
    pub fn new(
        gl: &glow::Context,
        assets: &Assets,
        block_registry: &BlockRegistry,
    ) -> Result<BlockRenderer, failure::Error> {
        // Normal rendering
        let program = ShaderProgram::load(
            gl,
            assets,
            "shaders/block.vert",
            "shaders/block.frag",
            "block".to_string(),
        )?;
        program.set_used(gl);
        let model_uniform = program.get_uniform(gl, "Model");
        let view_uniform = program.get_uniform(gl, "View");
        let projection_uniform = program.get_uniform(gl, "Projection");
        let z_offset_uniform = program.get_uniform(gl, "zOffset");
        if let Some(uniform) = program.get_uniform(gl, "blockTextures") {
            program.set_uniform_1i(gl, &uniform, 0);
        }
        // Build a list of texture file names sorted by texture array index
        let texture_paths = block_registry.texture_paths_sorted(assets);
        debug!("Loading {} block texture files..", texture_paths.len());

        let block_textures = MyTexture::load_array(
            texture_paths,
            TextureFormat::SRGBA8,
            TextureWrap::None,
            TextureFilter::MipMapNearest,
            gl,
        )
        .unwrap();

        // Translucent rendering
        let t_program = ShaderProgram::load(
            gl,
            assets,
            "shaders/translucent.vert",
            "shaders/translucent.frag",
            "translucent".to_string(),
        )?;
        t_program.set_used(gl);
        let t_model_uniform = t_program.get_uniform(gl, "Model");
        let t_view_uniform = t_program.get_uniform(gl, "View");
        let t_projection_uniform = t_program.get_uniform(gl, "Projection");
        let t_z_offset_uniform = t_program.get_uniform(gl, "zOffset");
        let light_dir_uniform = t_program.get_uniform(gl, "sunLightDirection");
        let light_col_uniform = t_program.get_uniform(gl, "sunLightColor");
        let fog_col_uniform = t_program.get_uniform(gl, "fogColor");
        let fog_start_uniform = t_program.get_uniform(gl, "fogStart");
        let fog_end_uniform = t_program.get_uniform(gl, "fogEnd");
        if let Some(uniform) = t_program.get_uniform(gl, "blockTextures") {
            t_program.set_uniform_1i(gl, &uniform, 0);
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
            t_z_offset_uniform,
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
        gl: &glow::Context,
        model: Mat4,
        camera: &PerspectiveCamera,
        max_range: i16,
        center_col: &ChunkColumnPos,
        render_lines: bool,
        out_of_range: &mut HashSet<ChunkColumnPos>,
    ) {
        self.program.set_used(gl);
        if let Some(uniform) = &self.model_uniform {
            self.program.set_uniform_matrix_4fv(gl, &uniform, &model);
        }
        if let Some(uniform) = &self.view_uniform {
            self.program
                .set_uniform_matrix_4fv(gl, &uniform, camera.get_view());
        }
        if let Some(uniform) = &self.projection_uniform {
            self.program
                .set_uniform_matrix_4fv(gl, &uniform, camera.get_projection());
        }
        self.block_textures.bind_at(gl, 0);
        unsafe {
            gl.enable(glow::CULL_FACE);
            gl.enable(glow::DEPTH_TEST);
            gl.disable(glow::BLEND);
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
                    gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
                }
            }

            let duration = render_instant - mesh.creation_instant;
            let duration_s =
                duration.as_secs() as f32 + duration.subsec_nanos() as f32 / 1_000_000_000.0;
            if mesh.animate && duration_s < 1.0 {
                if let Some(uniform) = &self.z_offset_uniform {
                    self.program
                        .set_uniform_1f(gl, &uniform, 0.5 + duration_s.powf(0.5) * 0.5);
                }
            }
            mesh.render(gl);
            if mesh.animate && duration_s < 1.0 {
                if let Some(uniform) = &self.z_offset_uniform {
                    self.program.set_uniform_1f(gl, &uniform, 1.0);
                }
            }
            if render_lines {
                unsafe {
                    gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
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
        gl: &glow::Context,
        model: Mat4,
        camera: &PerspectiveCamera,
        sun_dir: &Vec3,
        sun_col: &Vec3,
        fog_col: &Vec3,
        fog_start: f32,
        fog_end: f32,
    ) {
        self.t_program.set_used(gl);
        if let Some(uniform) = &self.t_model_uniform {
            self.t_program.set_uniform_matrix_4fv(gl, &uniform, &model);
        }
        if let Some(uniform) = &self.t_view_uniform {
            self.t_program
                .set_uniform_matrix_4fv(gl, &uniform, camera.get_view());
        }
        if let Some(uniform) = &self.t_projection_uniform {
            self.t_program
                .set_uniform_matrix_4fv(gl, &uniform, camera.get_projection());
        }
        if let Some(uniform) = &self.light_dir_uniform {
            self.t_program.set_uniform_3f(gl, &uniform, sun_dir);
        }
        if let Some(uniform) = &self.light_col_uniform {
            self.t_program.set_uniform_3f(gl, &uniform, sun_col);
        }
        if let Some(uniform) = &self.fog_col_uniform {
            self.t_program.set_uniform_3f(gl, &uniform, fog_col);
        }
        if let Some(uniform) = &self.fog_start_uniform {
            self.t_program.set_uniform_1f(gl, &uniform, fog_start);
        }
        if let Some(uniform) = &self.fog_end_uniform {
            self.t_program.set_uniform_1f(gl, &uniform, fog_end);
        }
        self.block_textures.bind_at(gl, 0);
        unsafe {
            gl.disable(glow::CULL_FACE);
            gl.enable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
            gl.enable(glow::FRAMEBUFFER_SRGB);
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
                    self.t_program
                        .set_uniform_1f(gl, &uniform, 0.5 + duration_s.powf(0.5) * 0.5);
                }
            }
            mesh.render(gl);
            if mesh.animate && duration_s < 1.0 {
                if let Some(uniform) = &self.t_z_offset_uniform {
                    self.t_program.set_uniform_1f(gl, &uniform, 1.0);
                }
            }
            vertex_count = vertex_count + mesh.vertex_count as usize;
            mesh_count = mesh_count + 1;
        }
        unsafe {
            gl.enable(glow::CULL_FACE);
            gl.disable(glow::FRAMEBUFFER_SRGB);
        }
        self.triangle_count = vertex_count as usize / 3;
        self.mesh_count = mesh_count;
    }

    pub fn get_stats(&self) -> (usize, usize) {
        (self.mesh_count, self.triangle_count)
    }
}
