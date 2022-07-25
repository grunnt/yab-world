#![allow(dead_code)]
use crate::physics::physicsobject::*;
use crate::world::worldhandler::{Direction, WorldHandler};
use common::block::*;
use glm::*;
use log::*;
use std::collections::HashMap;

// An object is "adjacent" to ground if below this margin
const OBJECT_ADJACENCY_MARGIN: f32 = 0.0001;
// In case of collision we align with a small margin so that it does not collide but is still adjacent
const OBJECT_ADJACENCY_ALIGNMENT: f32 = OBJECT_ADJACENCY_MARGIN / 2.0;

const RUNNING_VELOCITY: f32 = 0.1;
const WALKING_VELOCITY: f32 = 0.05;
const JUMP_VELOCITY: f32 = 0.15;
const WATER_SLOWDOWN: f32 = 0.5;
const AUTO_JUMP_VELOCITY: f32 = 0.1;

pub struct Physics {
    gravity: f32,
    max_velocity: f32,
    next_object_handle: u32,
    objects: HashMap<u32, PhysicsObject>,
    water_block: Block,
}

impl Physics {
    pub fn new(block_registry: &BlockRegistry) -> Physics {
        Physics {
            gravity: 0.015,
            max_velocity: 100.0, // 0.8, // Should never be >= 1.0 to prevent tunneling
            next_object_handle: 0,
            objects: HashMap::new(),
            water_block: block_registry.block_kind_from_code("wtr"),
        }
    }

    pub fn new_object(&mut self, x: f32, y: f32, z: f32, size: Vec3) -> u32 {
        let object = PhysicsObject {
            position: Vec3::new(x, y, z),
            velocity: Vec3::zeros(),
            facing: Vec3::y(),
            size,
            state: PhysicsObjectState::Flying,
            on_ground: false,
            was_in_water: false,
            in_water: false,
            controls: PhysicsObjectControls::new(),
            gravity_factor: 1.0,
            colliding: true,
        };
        let object_handle = self.next_object_handle;
        self.objects.insert(object_handle, object);
        self.next_object_handle = self.next_object_handle + 1;
        object_handle
    }

    pub fn get_object(&self, handle: u32) -> &PhysicsObject {
        &self.objects.get(&handle).unwrap()
    }

    pub fn get_object_mut(&mut self, handle: u32) -> &mut PhysicsObject {
        self.objects.get_mut(&handle).unwrap()
    }

    pub fn get_object_position(&self, handle: u32) -> Vec3 {
        self.objects.get(&handle).unwrap().position
    }

    pub fn get_object_state(&self, handle: u32) -> &PhysicsObjectState {
        &self.objects.get(&handle).unwrap().state
    }

    pub fn set_object_controls(&mut self, handle: u32, controls: PhysicsObjectControls) {
        self.objects.get_mut(&handle).unwrap().controls = controls;
    }

    pub fn set_object_position(&mut self, handle: u32, position: &Vec3) {
        self.objects.get_mut(&handle).unwrap().position = *position;
    }

    pub fn set_object_facing(&mut self, handle: u32, facing: &Vec3) {
        self.objects.get_mut(&handle).unwrap().facing = facing.clone();
    }

    pub fn set_object_gravity_factor(&mut self, handle: u32, gravity_factor: f32) {
        self.objects.get_mut(&handle).unwrap().gravity_factor = gravity_factor;
    }

    pub fn set_object_velocity(&mut self, handle: u32, velocity: &Vec3) {
        let object = self.objects.get_mut(&handle).unwrap();
        object.velocity = *velocity;
        object.state = PhysicsObjectState::Flying;
    }

    pub fn set_object_colliding(&mut self, handle: u32, colliding: bool) {
        self.objects.get_mut(&handle).unwrap().colliding = colliding;
    }

    pub fn is_object_in_water(&mut self, handle: u32, offset: Vec3, world: &WorldHandler) -> bool {
        let pos = self.get_object_position(handle) + offset;
        world
            .chunks
            .get_block(pos.x as i16, pos.y as i16, pos.z as i16)
            .kind()
            == self.water_block
    }

    pub fn is_occopied_by_body(
        &mut self,
        wbx: i16,
        wby: i16,
        wbz: i16,
        world: &mut WorldHandler,
    ) -> bool {
        // Temporarily set block to do detection
        let old_block = world.chunks.get_block(wbx, wby, wbz);
        world.chunks.set_block(wbx, wby, wbz, BEDROCK_BLOCK);
        for (_, object) in &mut self.objects {
            let p1 = object.position - object.size / 2.0;
            let p2 = object.position + object.size / 2.0;
            let colliding = world.box_collisions(
                p1,
                p2,
                Vec3::new(std::f32::EPSILON, std::f32::EPSILON, std::f32::EPSILON),
            );
            if colliding.contains_key(&Direction::NONE) {
                // This means that the collision detection ray(s) started inside our temporary block
                world.chunks.set_block(wbx, wby, wbz, old_block);
                return true;
            }
        }
        world.chunks.set_block(wbx, wby, wbz, old_block);
        false
    }

    pub fn step(&mut self, world: &WorldHandler) {
        for (handle, object) in &mut self.objects {
            if object.colliding {
                // Determine object corner points
                let p1 = object.position - object.size / 2.0;
                let p2 = object.position + object.size / 2.0;
                let p_up_1 = p1 + Vec3::new(0.0, 0.0, 1.0);
                let p_up_2 = p2 + Vec3::new(0.0, 0.0, 1.0);
                // Do collision detection using ray casting at the point where the object would end up after applying velocity
                if object.velocity != Vec3::zeros() {
                    // Cast rays from object to world and see where the closest one hits
                    let collisions = world.box_collisions(p1, p2, object.velocity);
                    if !collisions.is_empty() {
                        let collisions_1_block_up =
                            world.box_collisions(p_up_1, p_up_2, object.velocity);
                        for (dir, collision) in collisions {
                            match dir {
                                Direction::XP => {
                                    if collision.hit_delta.x.abs() > OBJECT_ADJACENCY_MARGIN {
                                        object.position.x = object.position.x
                                            + collision.hit_delta.x
                                            - OBJECT_ADJACENCY_ALIGNMENT;
                                    }
                                    if collisions_1_block_up.is_empty() && !object.controls.slower {
                                        object.velocity.z = AUTO_JUMP_VELOCITY;
                                    }
                                    object.velocity.x = 0.0;
                                }
                                Direction::XM => {
                                    if collision.hit_delta.x.abs() > OBJECT_ADJACENCY_MARGIN {
                                        object.position.x = object.position.x
                                            + collision.hit_delta.x
                                            + OBJECT_ADJACENCY_ALIGNMENT;
                                    }
                                    if collisions_1_block_up.is_empty() && !object.controls.slower {
                                        object.velocity.z = AUTO_JUMP_VELOCITY;
                                    }
                                    object.velocity.x = 0.0;
                                }
                                Direction::YP => {
                                    if collision.hit_delta.y.abs() > OBJECT_ADJACENCY_MARGIN {
                                        object.position.y = object.position.y
                                            + collision.hit_delta.y
                                            - OBJECT_ADJACENCY_ALIGNMENT;
                                    }
                                    if collisions_1_block_up.is_empty() && !object.controls.slower {
                                        object.velocity.z = AUTO_JUMP_VELOCITY;
                                    }
                                    object.velocity.y = 0.0;
                                }
                                Direction::YM => {
                                    if collision.hit_delta.y.abs() > OBJECT_ADJACENCY_MARGIN {
                                        object.position.y = object.position.y
                                            + collision.hit_delta.y
                                            + OBJECT_ADJACENCY_ALIGNMENT;
                                    }
                                    if collisions_1_block_up.is_empty() && !object.controls.slower {
                                        object.velocity.z = AUTO_JUMP_VELOCITY;
                                    }
                                    object.velocity.y = 0.0;
                                }
                                Direction::ZP => {
                                    if collision.hit_delta.z.abs() > OBJECT_ADJACENCY_MARGIN {
                                        object.position.z = object.position.z
                                            + collision.hit_delta.z
                                            - OBJECT_ADJACENCY_ALIGNMENT;
                                    }
                                    if object.velocity.z > 0.0 {
                                        object.velocity.z = 0.0;
                                    }
                                }
                                Direction::ZM => {
                                    let delta_z = collision.hit_delta.z.abs();
                                    if delta_z > OBJECT_ADJACENCY_MARGIN {
                                        object.position.z = object.position.z
                                            + collision.hit_delta.z
                                            + OBJECT_ADJACENCY_ALIGNMENT;
                                    }
                                    if object.velocity.z < 0.0 {
                                        object.velocity.z = 0.0;
                                    }
                                }
                                Direction::NONE => {
                                    // This is bad, should never happen. But we'll try to recover anyway by moving the player up until there's no collision.
                                    warn!(
                                    "body inside world - collision = {:?}, p1 = {:?}, p2 = {:?} - moving UP",
                                    collision, p1, p2
                                );
                                    let mut bad = true;
                                    while bad {
                                        object.position.z = object.position.z + 10.0;
                                        object.velocity = Vec3::zeros();
                                        let p1 = object.position - object.size / 2.0;
                                        let p2 = object.position + object.size / 2.0;
                                        if world.box_collisions(p1, p2, object.velocity).is_empty()
                                        {
                                            bad = false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Update position
                object.velocity = object.velocity * 0.99;
                object.velocity = limit(object.velocity, self.max_velocity);
            }

            // Safety check: nothing may move outside of a loaded chunks (this assumes a chunk is not unloaded)
            if world.chunks.is_pos_in_loaded_chunk(
                object.position.x + object.velocity.x,
                object.position.y + object.velocity.y,
            ) {
                object.position = object.position + object.velocity;
            }

            // Safety check: nothing may ever be below 0
            if object.position.z < 0.0 {
                warn!("Object {} was below z = 0", handle);
                object.position.z = 0.0;
                object.velocity.z = 0.0;
            }

            if object.colliding {
                // Determine object corner points for new position
                let p1 = object.position - object.size / 2.0;
                let p2 = object.position + object.size / 2.0;

                // Determine whether the object is on the ground
                if let Some(_) = world
                    .box_collisions(p1, p2, -Vec3::z() * OBJECT_ADJACENCY_MARGIN)
                    .get(&Direction::ZM)
                {
                    object.on_ground = true;
                } else {
                    object.on_ground = false;
                }

                // Determine whether the object is in the water
                object.was_in_water = object.in_water;
                object.in_water = world
                    .chunks
                    .get_block(
                        object.position.x as i16,
                        object.position.y as i16,
                        object.position.z as i16,
                    )
                    .kind()
                    == self.water_block;

                // Add some buoyancy
                if object.in_water {
                    object.velocity.z =
                        object.velocity.z + (self.gravity * object.gravity_factor * 0.9);
                }

                // Update object state
                match object.state {
                    PhysicsObjectState::Stopped => {
                        object.velocity.x = 0.0;
                        object.velocity.y = 0.0;
                        if !object.on_ground {
                            object.state = PhysicsObjectState::Flying;
                        } else if object.controls.is_moving() {
                            object.state = PhysicsObjectState::Moving;
                        } else if object.controls.is_jumping() {
                            object.state = PhysicsObjectState::Flying;
                            object.velocity.z = object.velocity.z + JUMP_VELOCITY;
                            object.position.z = object.position.z + OBJECT_ADJACENCY_MARGIN;
                        }
                    }
                    PhysicsObjectState::Moving => {
                        if !object.controls.is_moving() {
                            object.state = PhysicsObjectState::Stopped;
                            object.velocity.x = 0.0;
                            object.velocity.y = 0.0;
                        } else if object.controls.is_jumping() {
                            object.state = PhysicsObjectState::Flying;
                            object.velocity.z = object.velocity.z + JUMP_VELOCITY * 2.0; // Jump higher when running
                            object.position.z = object.position.z + OBJECT_ADJACENCY_MARGIN;
                        } else {
                            set_velocity_from_controls(object);
                        }
                        if !object.on_ground {
                            object.state = PhysicsObjectState::Flying;
                        }
                    }
                    PhysicsObjectState::Flying => {
                        object.velocity.z =
                            object.velocity.z - (self.gravity * object.gravity_factor);
                        set_velocity_from_controls(object);
                        if object.on_ground {
                            if object.controls.is_moving() {
                                object.state = PhysicsObjectState::Moving;
                                object.velocity.z = 0.0;
                            } else {
                                object.state = PhysicsObjectState::Stopped;
                                object.velocity = Vec3::zeros();
                            }
                        }
                    }
                }
            } else {
                set_velocity_from_controls(object);
            }
        }
    }
}

fn limit(vector: Vec3, max_length: f32) -> Vec3 {
    let length = vector.norm();
    if (length > max_length) && (length > 0.0) {
        let ratio = max_length / length;
        vector * ratio
    } else {
        vector
    }
}

fn set_velocity_from_controls(object: &mut PhysicsObject) {
    if object.colliding {
        // Normal movement
        let mut speed = if object.controls.slower {
            WALKING_VELOCITY
        } else {
            RUNNING_VELOCITY
        };
        if object.in_water {
            speed = speed * WATER_SLOWDOWN;
        }
        if !object.controls.is_moving() && object.state != PhysicsObjectState::Flying {
            object.velocity.x = 0.0;
            object.velocity.y = 0.0;
        } else {
            if object.controls.forward {
                if object.in_water {
                    let velocity = object.facing * speed;
                    object.velocity = Vec3::new(velocity.x, velocity.y, velocity.z);
                } else {
                    let horizontal_velocity = object.facing.xy().normalize() * speed;
                    object.velocity = Vec3::new(
                        horizontal_velocity.x,
                        horizontal_velocity.y,
                        object.velocity.z,
                    );
                }
            } else if object.controls.backward {
                let horizontal_velocity = -object.facing.xy().normalize() * speed;
                object.velocity = Vec3::new(
                    horizontal_velocity.x,
                    horizontal_velocity.y,
                    object.velocity.z,
                );
            }
            let perpendicular = Vec2::new(-object.facing.y, object.facing.x).normalize();
            if object.controls.left && !object.controls.right {
                let horizontal_velocity = perpendicular * speed;
                object.velocity = Vec3::new(
                    horizontal_velocity.x,
                    horizontal_velocity.y,
                    object.velocity.z,
                );
            } else if object.controls.right && !object.controls.left {
                let horizontal_velocity = -perpendicular * speed;
                object.velocity = Vec3::new(
                    horizontal_velocity.x,
                    horizontal_velocity.y,
                    object.velocity.z,
                );
            }
            if object.in_water {
                if object.controls.is_jumping() {
                    object.velocity = Vec3::new(object.velocity.x, object.velocity.y, speed);
                } else if object.controls.is_ducking() {
                    object.velocity = Vec3::new(object.velocity.x, object.velocity.y, -speed);
                }
            }
        }
    } else {
        // Move without collisions or gravity (god mode)
        let speed = if object.controls.slower {
            WALKING_VELOCITY * 2.0
        } else {
            RUNNING_VELOCITY * 3.0
        };
        if !object.controls.is_moving() {
            object.velocity.x = 0.0;
            object.velocity.y = 0.0;
        }
        if !object.controls.up && !object.controls.down {
            object.velocity.z = 0.0;
        }
        if object.controls.forward {
            let horizontal_velocity = object.facing.xy().normalize() * speed;
            object.velocity = Vec3::new(
                horizontal_velocity.x,
                horizontal_velocity.y,
                object.velocity.z,
            );
        } else if object.controls.backward {
            let horizontal_velocity = -object.facing.xy().normalize() * speed;
            object.velocity = Vec3::new(
                horizontal_velocity.x,
                horizontal_velocity.y,
                object.velocity.z,
            );
        }
        let perpendicular = Vec2::new(-object.facing.y, object.facing.x).normalize();
        if object.controls.left && !object.controls.right {
            let horizontal_velocity = perpendicular * speed;
            object.velocity = Vec3::new(
                horizontal_velocity.x,
                horizontal_velocity.y,
                object.velocity.z,
            );
        } else if object.controls.right && !object.controls.left {
            let horizontal_velocity = -perpendicular * speed;
            object.velocity = Vec3::new(
                horizontal_velocity.x,
                horizontal_velocity.y,
                object.velocity.z,
            );
        }
        if object.controls.up && !object.controls.down {
            object.velocity = Vec3::new(object.velocity.x, object.velocity.y, speed);
        } else if object.controls.down && !object.controls.up {
            object.velocity = Vec3::new(object.velocity.x, object.velocity.y, -speed);
        }
    }
}
