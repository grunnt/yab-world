use std::collections::HashMap;

use super::*;
use crate::*;
use nalgebra_glm::{Mat4, Vec3};
use rand::{prelude::ThreadRng, Rng};

pub type EmitterHandle = usize;

pub struct ParticleSystem {
    texture: TextureArray,
    program: ShaderProgram,
    projection_uniform: UniformLocation,
    view_uniform: UniformLocation,
    viewport_height_uniform: UniformLocation,
    vbo: VBO,
    vao: VAO,
    vertices: Vec<ParticleVertex>,
    particles: Vec<Particle>,
    emitters: HashMap<EmitterHandle, Emitter>,
    next_emitter_handle: EmitterHandle,
    positions: HashMap<ParticlePositionHandle, Vec3>,
    next_particle_position_handle: ParticlePositionHandle,
    rng: ThreadRng,
    last_id: usize,
}

impl ParticleSystem {
    pub fn new(gl: &glow::Context, assets: &Assets, texture: TextureArray) -> Self {
        let program = ShaderProgram::load(
            gl,
            assets,
            "shaders/particle.vert",
            "shaders/particle.frag",
            "particle".to_string(),
        )
        .unwrap();

        program.set_used(gl);
        let projection_uniform = program.get_uniform(gl, "projection").unwrap();
        let view_uniform = program.get_uniform(gl, "view").unwrap();
        let viewport_height_uniform = program.get_uniform(gl, "viewport_height").unwrap();
        if let Some(uniform) = program.get_uniform(gl, "textures") {
            program.set_uniform_1i(gl, &uniform, 0);
        }
        // Vertex array and object
        let vbo = VBO::new(gl);
        let vao = VAO::new(gl);
        vao.bind(gl);
        vbo.bind(gl);
        ParticleVertex::vertex_attrib_pointers(gl);

        ParticleSystem {
            program,
            projection_uniform,
            view_uniform,
            viewport_height_uniform,
            vbo,
            vao,
            vertices: Vec::new(),
            texture,
            particles: Vec::new(),
            emitters: HashMap::new(),
            next_emitter_handle: 1,
            positions: HashMap::new(),
            next_particle_position_handle: 1,
            rng: rand::thread_rng(),
            last_id: 0,
        }
    }

    pub fn emitter(
        &mut self,
        position: ParticlePosition,
        target: ParticlePosition,
        definition: EmitterDef,
    ) -> EmitterHandle {
        let life_time_s = definition.duration;
        let emitter = Emitter {
            active: true,
            position,
            target,
            definition,
            accumulated_s: 0.0,
            life_time_s,
        };
        let handle = self.next_emitter_handle;
        self.next_emitter_handle += 1;
        self.emitters.insert(handle, emitter);
        handle
    }

    pub fn emitter_mut(&mut self, handle: EmitterHandle) -> Option<&mut Emitter> {
        self.emitters.get_mut(&handle)
    }

    pub fn new_position_handle(&mut self) -> ParticlePositionHandle {
        let handle = self.next_particle_position_handle;
        self.next_particle_position_handle += 1;
        self.positions.insert(handle, Vec3::zeros());
        handle
    }

    pub fn update_position_handle(&mut self, handle: ParticlePositionHandle, new_position: Vec3) {
        self.positions.insert(handle, new_position);
    }

    pub fn remove_position_handle(&mut self, handle: ParticlePositionHandle) {
        self.positions.remove(&handle);
    }

    pub fn update(&mut self, delta_s: f32) {
        // Remove inactive particles
        self.particles.retain(|p| p.lifetime > 0.0);

        // Emit new particles
        let mut new_particles = Vec::new();
        for (_, emitter) in &mut self.emitters {
            if emitter.is_active() {
                emitter.accumulated_s += delta_s;
                while emitter.accumulated_s > emitter.definition.particle_interval_s {
                    emitter.accumulated_s -= emitter.definition.particle_interval_s;
                    let speed = emitter.definition.velocity.min
                        + self.rng.gen::<f32>()
                            * (emitter.definition.velocity.max - emitter.definition.velocity.min);
                    let pitch = if emitter.definition.spread_angle > 0.0 {
                        self.rng.gen_range(
                            -emitter.definition.spread_angle,
                            emitter.definition.spread_angle,
                        ) + emitter.definition.pitch
                    } else {
                        0.0
                    };
                    let yaw = if emitter.definition.spread_angle > 0.0 {
                        self.rng.gen_range(
                            -emitter.definition.spread_angle,
                            emitter.definition.spread_angle,
                        ) + emitter.definition.yaw
                    } else {
                        0.0
                    };
                    let velocity = glm::rotate_vec3(&Vec3::y(), pitch, &Vec3::z());
                    let velocity = glm::rotate_vec3(&velocity, yaw, &Vec3::x()) * speed;
                    let lifetime = self
                        .rng
                        .gen_range(emitter.definition.life.min, emitter.definition.life.max);
                    let emitter_position = match emitter.position {
                        ParticlePosition::Fixed(pos) => pos,
                        ParticlePosition::Handle(handle) => *self.positions.get(&handle).unwrap(),
                        ParticlePosition::None => panic!("Emitter must have position"),
                    };
                    let position = if let Some(area) = emitter.definition.start_area {
                        emitter_position
                            + Vec3::new(
                                area.x * self.rng.gen_range(-1.0, 1.0),
                                area.y * self.rng.gen_range(-1.0, 1.0),
                                area.z * self.rng.gen_range(-1.0, 1.0),
                            )
                    } else {
                        emitter_position
                    };
                    new_particles.push(Particle {
                        position,
                        velocity,
                        target: emitter.target,
                        texture_layer: emitter.definition.texture_layers
                            [self.last_id % emitter.definition.texture_layers.len()],
                        size: self
                            .rng
                            .gen_range(emitter.definition.size.min, emitter.definition.size.max),
                        lifetime,
                        total_lifetime: lifetime,
                        id: self.last_id,
                    });
                    self.last_id += 1;
                }
            }
        }
        self.particles.append(&mut new_particles);
        // Update particles
        for particle in &mut self.particles {
            match particle.target {
                ParticlePosition::Fixed(position) => {
                    // Move towards a target
                    let speed = particle.velocity.norm() * delta_s;
                    let velocity = (position - particle.position).normalize() * speed;
                    particle.position += velocity;
                    if particle.position.metric_distance(&position) < speed {
                        // Close to target so discard this particle
                        particle.lifetime = 0.0;
                    }
                }
                ParticlePosition::Handle(handle) => {
                    let target = self.positions.get(&handle).unwrap();
                    // Move towards a target
                    let speed = particle.velocity.norm() * delta_s;
                    let velocity = (target - particle.position).normalize() * speed;
                    particle.position += velocity;
                    if particle.position.metric_distance(&target) < speed {
                        // Close to target so discard this particle
                        particle.lifetime = 0.0;
                    }
                }
                ParticlePosition::None => {
                    // Linear movement
                    particle.position += particle.velocity * delta_s;
                    particle.lifetime -= delta_s;
                }
            }
        }
        // Update emitter timers
        for (_, emitter) in &mut self.emitters {
            if !emitter.definition.continuous {
                emitter.life_time_s -= delta_s;
            }
        }
        // Remove inactive emitters
        self.emitters
            .retain(|_, e| e.definition.continuous || e.life_time_s > 0.0);
    }

    pub fn draw(
        &mut self,
        gl: &glow::Context,
        view: &Mat4,
        projection: &Mat4,
        viewport_height: f32,
    ) {
        // Generate vertices for the particles
        for particle in &self.particles {
            self.vertices.push(ParticleVertex::new(
                particle.position.x,
                particle.position.y,
                particle.position.z,
                particle.texture_layer,
                particle.size,
                particle.lifetime / particle.total_lifetime,
            ));
        }

        if self.vertices.is_empty() {
            return;
        }

        // Upload the vertices
        self.vbo.bind(gl);
        // TODO set to null
        // self.vbo
        //     .stream_draw_data_null::<ParticleVertex>(self.vertices.len());
        self.vbo
            .stream_draw_data::<ParticleVertex>(gl, &self.vertices);
        self.vbo.unbind(gl);

        // Render the particles
        self.program.set_used(gl);
        self.program
            .set_uniform_matrix_4fv(gl, &self.view_uniform, view);
        self.program
            .set_uniform_matrix_4fv(gl, &self.projection_uniform, projection);
        self.program
            .set_uniform_1f(gl, &self.viewport_height_uniform, viewport_height);
        self.vao.bind(gl);
        self.texture.texture().bind_at(gl, 0);
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.enable(glow::VERTEX_PROGRAM_POINT_SIZE);
            gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
            gl.draw_arrays(glow::POINTS, 0, self.vertices.len() as i32);
        }

        // Clear the vertex buffer for the next frame
        self.vertices.clear();
    }

    pub fn texture_array(&self) -> &TextureArray {
        &self.texture
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }
}

#[derive(Clone, Debug)]
pub struct Effect {
    pub emitters: Vec<EmitterDef>,
}

#[derive(Clone, Debug)]
pub struct Emitter {
    pub active: bool,
    pub position: ParticlePosition,
    pub target: ParticlePosition,
    pub definition: EmitterDef,
    pub accumulated_s: f32,
    pub life_time_s: f32,
}

impl Emitter {
    pub fn is_active(&self) -> bool {
        self.active && (self.definition.continuous || self.life_time_s > 0.0)
    }
}

#[derive(Clone, Debug)]
pub struct EmitterDef {
    pub pitch: f32,
    pub yaw: f32,
    pub spread_angle: f32,
    pub start_area: Option<Vec3>,
    pub delay: f32,
    pub duration: f32,
    pub continuous: bool,
    pub particle_interval_s: f32,
    pub size: Range,
    pub life: Range,
    pub velocity: Range,
    pub texture_layers: Vec<f32>,
}

pub type ParticlePositionHandle = usize;

#[derive(Copy, Clone, Debug)]
pub enum ParticlePosition {
    Fixed(Vec3),
    Handle(ParticlePositionHandle),
    None,
}

#[derive(Clone, Debug)]
pub struct Particle {
    pub id: usize,
    pub position: Vec3,
    pub target: ParticlePosition,
    pub velocity: Vec3,
    pub texture_layer: f32,
    pub size: f32,
    pub lifetime: f32,
    pub total_lifetime: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct Range {
    pub min: f32,
    pub max: f32,
}

impl Range {
    pub fn new(min: f32, max: f32) -> Self {
        Range { min, max }
    }
}

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct ParticleVertex {
    #[location = "0"]
    pub position: data::f32_f32_f32,
    #[location = "1"]
    pub layer_size_life: data::f32_f32_f32,
}

impl ParticleVertex {
    pub fn new(x: f32, y: f32, z: f32, layer: f32, size: f32, life: f32) -> Self {
        ParticleVertex {
            position: (x, y, z).into(),
            layer_size_life: (layer, size, life).into(),
        }
    }
}
