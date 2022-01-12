use crate::render::*;
use crate::world::light::LightHandler;
use common::block::*;
use common::chunk::chunk_buffer::ChunkBuffer;
use common::chunk::*;
use common::comms::*;
use crossbeam::channel::*;
use crossbeam::unbounded;
use failure;
use gamework::profile::Profile;
use gamework::video::FrustumChecker;
use log::*;
use nalgebra_glm::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::f32;
use std::io::Cursor;
use std::thread;
use std::time::Instant;

pub struct WorldHandler {
    pub chunks: ChunkBuffer,
    pub render_range: usize,
    pub prio_render_range: i16,
    pub center_col: ChunkColumnPos,
    column_rx: Receiver<ChunkColumn>,
    vertices_rx: Receiver<(ChunkPos, Vec<BlockVertex>, Vec<BlockVertex>)>,
    status_rx: Receiver<(ChunkColumnPos, ColumnStatus)>,
    block_registry: BlockRegistry,
    request_candidates: VecDeque<ChunkColumnPos>,
}

impl WorldHandler {
    pub fn new(
        render_range: usize,
        starting_column: ChunkColumnPos,
        col_receiver: Receiver<ServerMessage>,
        block_registry: BlockRegistry,
    ) -> Result<WorldHandler, failure::Error> {
        let (column_tx, column_rx): (Sender<ChunkColumn>, Receiver<ChunkColumn>) = unbounded();
        let (vertices_tx, vertices_rx) = unbounded();
        let (status_tx, status_rx) = unbounded();

        let mut buffer = ChunkBuffer::new();
        let worldmesher = WorldMesher::new(block_registry.clone());
        let block_registry_clone = block_registry.clone();

        let mut loop_profile = Profile::new(1);
        let mut light_profile = Profile::new(1);
        let mut mesh_profile = Profile::new(1);
        let mut server_profile = Profile::new(1);
        let mut last_message = Instant::now();

        thread::Builder::new()
            .name("world_handler".to_string())
            .spawn(move || {
                'handler: loop {
                    loop_profile.start();
                    server_profile.start();
                    // Block until we receive a new column from the server
                    let message = match col_receiver.recv() {
                        Err(e) => {
                            debug!("World handler shutting down: {}", e);
                            break 'handler;
                        }
                        Ok(message) => message,
                    };
                    server_profile.end();
                    // Handle received message
                    match message {
                        // Store the received column and propagate sunlight
                        ServerMessage::ChunkColumn { col, block_data } => {
                            match status_tx.send((col, ColumnStatus::Received)) {
                                Err(e) => {
                                    debug!("World handler shutting down: {}", e);
                                    break 'handler;
                                }
                                _ => {}
                            };
                            let mut chunks = Vec::new();
                            for z in 0..WORLD_HEIGHT_CHUNKS {
                                let mut bytes_reader = Cursor::new(&block_data[z]);
                                let blocks = Vec::rle_decode_from(&mut bytes_reader).unwrap();
                                let chunk = Chunk {
                                    pos: ChunkPos::new(col.x, col.y, z as i16),
                                    blocks,
                                };
                                assert!(chunk.is_initialized());
                                chunks.push(chunk);
                            }
                            buffer.store_column(ChunkColumn::new(
                                col,
                                ColumnStatus::Stored,
                                chunks,
                            ));
                        }
                        _ => {
                            panic!("Cannot handle {:?}", message);
                        }
                    }
                    // Do light propagation for all columns with all neighbours stored
                    light_profile.start();
                    let mut propagation_columns = Vec::new();
                    for (col, column) in &buffer.columns {
                        if column.status() == ColumnStatus::Stored
                            && buffer.are_all_neighbours_stored(*col)
                        {
                            // This column needs to have light propagated
                            propagation_columns.push(col.clone());
                        }
                    }
                    for col in propagation_columns {
                        // Propagate light in all chunks of this column
                        for z in 0..WORLD_HEIGHT_CHUNKS {
                            buffer.propagate_chunk_lights(
                                ChunkPos::new(col.x, col.y, z as i16),
                                &mut HashSet::new(),
                                &block_registry_clone,
                            );
                        }
                        // Mark the column as propagated (ready for meshing)
                        buffer
                            .get_mut_column(col.x, col.y)
                            .unwrap()
                            .set_status(ColumnStatus::Propagated);
                        // Send a copy to the main thread, e.g. for collision detection
                        match column_tx.send(buffer.get_column_clone(col.x, col.y).unwrap()) {
                            Err(e) => {
                                debug!("World handler shutting down: {}", e);
                                break 'handler;
                            }
                            _ => {}
                        };
                    }
                    light_profile.end();
                    // Do meshing for all columns with neighbours light propagated
                    mesh_profile.start();
                    let mut mesh_columns = Vec::new();
                    for (col, column) in &buffer.columns {
                        let all_propagated = buffer.are_all_neighbours_propagated(column.col);
                        if column.status() == ColumnStatus::Propagated && all_propagated {
                            mesh_columns.push(col.clone());
                        }
                    }
                    for col in &mesh_columns {
                        let col_vertices = worldmesher.mesh_column(col, &buffer);
                        let mut z = 0;
                        for (vertices, translucent_vertices) in col_vertices {
                            if vertices.len() > 0 || translucent_vertices.len() > 0 {
                                match vertices_tx.send((
                                    ChunkPos::new(col.x, col.y, z as i16),
                                    vertices,
                                    translucent_vertices,
                                )) {
                                    Err(e) => {
                                        debug!("World handler shutting down: {}", e);
                                        break 'handler;
                                    }
                                    _ => {}
                                }
                            }
                            z = z + 1;
                        }
                        match status_tx.send((col.clone(), ColumnStatus::Meshed)) {
                            Err(e) => {
                                debug!("World handler shutting down: {}", e);
                                break 'handler;
                            }
                            _ => {}
                        }
                        buffer
                            .get_mut_column(col.x, col.y)
                            .unwrap()
                            .set_status(ColumnStatus::Meshed);
                    }
                    mesh_profile.end();

                    loop_profile.end();

                    let time_since_last = Instant::now() - last_message;
                    if time_since_last.as_secs() >= 1 {
                        loop_profile.frame();
                        light_profile.frame();
                        mesh_profile.frame();
                        server_profile.frame();
                        debug!(
                            "Loop {}, server {}, mesh {}, light {}",
                            loop_profile.max_ms,
                            server_profile.max_ms,
                            mesh_profile.max_ms,
                            light_profile.max_ms,
                        );
                        last_message = Instant::now();
                    }
                }
            })
            .unwrap();

        let chunks = ChunkBuffer::new();

        Ok(WorldHandler {
            render_range,
            prio_render_range: 4,
            center_col: starting_column,
            chunks,
            column_rx,
            vertices_rx,
            status_rx,
            block_registry,
            request_candidates: VecDeque::new(),
        })
    }

    pub fn set_buffer_position(&mut self, center_col: ChunkColumnPos) {
        self.center_col = center_col;
    }

    pub fn try_receive_status(&mut self) -> Option<(ChunkColumnPos, ColumnStatus)> {
        let result = self.status_rx.try_recv();
        if result.is_ok() {
            let (col, status) = result.unwrap();
            if let Some(column) = self.chunks.get_mut_column(col.x, col.y) {
                column.set_status(status);
            }
            Some((col, status))
        } else {
            None
        }
    }

    pub fn get_next_request(&mut self, frustum: Option<&FrustumChecker>) -> Option<ChunkColumnPos> {
        // Prioritize nearby columns
        let center_col = self.center_col.clone();
        for dx in -self.prio_render_range..self.prio_render_range + 1 {
            for dy in -self.prio_render_range..self.prio_render_range + 1 {
                let col = ChunkColumnPos::new(center_col.x + dx, center_col.y + dy);
                if !self.chunks.is_column_requested(&col) {
                    self.set_column_requested(&col);
                    return Some(col);
                }
            }
        }
        // Check the rest that are not too far and in the horizontal camera frustum
        let dst_sq = (self.render_range * self.render_range) as i16;
        let mut col_opt = None;
        for candidate in &self.request_candidates {
            if candidate.dist_squared_from(&center_col) <= dst_sq {
                if let Some(frustum) = frustum {
                    if is_column_inside_frustrum_xy(&frustum, candidate) {
                        col_opt = Some(*candidate);
                        break;
                    }
                } else {
                    col_opt = Some(*candidate);
                    break;
                }
            }
        }
        if let Some(col) = col_opt {
            self.set_column_requested(&col);
        }
        col_opt
    }

    /// Mark a chunk column as requested (i.e. awaiting receive from server)
    fn set_column_requested(&mut self, col: &ChunkColumnPos) {
        self.request_candidates.retain(|c| c != col);
        self.try_add_candidate(&col.xp());
        self.try_add_candidate(&col.xm());
        self.try_add_candidate(&col.yp());
        self.try_add_candidate(&col.ym());
        self.chunks.store_column(ChunkColumn::new(
            col.clone(),
            ColumnStatus::Requested,
            Vec::new(),
        ));
    }

    fn try_add_candidate(&mut self, col: &ChunkColumnPos) {
        if !self.chunks.is_column_requested(col) {
            self.request_candidates.push_back(col.clone());
        }
    }

    pub fn try_receive_columns(&mut self) {
        loop {
            let result = self.column_rx.try_recv();
            if result.is_ok() {
                let column = result.unwrap();
                self.store_column(column);
            } else {
                break;
            }
        }
    }

    pub fn remove_column(&mut self, col: &ChunkColumnPos) {
        self.chunks.columns.remove(&col);
    }

    pub fn try_receive_vertices(
        &mut self,
    ) -> Option<(ChunkPos, Vec<BlockVertex>, Vec<BlockVertex>)> {
        let result = self.vertices_rx.try_recv();
        if result.is_ok() {
            Some(result.unwrap())
        } else {
            None
        }
    }

    // Based on http://www.cse.chalmers.se/edu/year/2010/course/TDA361/grid.pdf
    // and https://github.com/andyhall/fast-voxel-raycast/blob/master/index.js
    pub fn cast_ray(
        &self,
        position: &Vec3,
        direction: &Vec3,
        max_distance: f32,
        hit_water: bool,
    ) -> Option<RayHit> {
        let mut t = 0.0;
        let mut wcx = position.x.floor();
        let mut wcy = position.y.floor();
        let mut iz = position.z.floor();
        let stepx = if direction.x > 0.0 { 1 } else { -1 };
        let stepy = if direction.y > 0.0 { 1 } else { -1 };
        let stepz = if direction.z > 0.0 { 1 } else { -1 };
        let direction = direction.normalize();
        let tx_delta = if direction.x == 0.0 {
            f32::MAX
        } else {
            (1.0 / direction.x).abs()
        };
        let ty_delta = if direction.y == 0.0 {
            f32::MAX
        } else {
            (1.0 / direction.y).abs()
        };
        let tz_delta = if direction.z == 0.0 {
            f32::MAX
        } else {
            (1.0 / direction.z).abs()
        };
        // Distance to take each step
        let x_dist = if stepx > 0 {
            wcx + 1.0 - position.x
        } else {
            position.x - wcx
        };
        let y_dist = if stepy > 0 {
            wcy + 1.0 - position.y
        } else {
            position.y - wcy
        };
        let z_dist = if stepz > 0 {
            iz + 1.0 - position.z
        } else {
            position.z - iz
        };
        // location of nearest voxel boundary, in units of t
        let mut tx_max = if tx_delta < f32::MAX {
            tx_delta * x_dist
        } else {
            f32::MAX
        };
        let mut ty_max = if ty_delta < f32::MAX {
            ty_delta * y_dist
        } else {
            f32::MAX
        };
        let mut tz_max = if tz_delta < f32::MAX {
            tz_delta * z_dist
        } else {
            f32::MAX
        };
        let mut stepped_index = -1;
        // Now walk along the ray from block edge to block edge
        while t < max_distance {
            let mut block = None;
            if let Some(chunk) = self
                .chunks
                .get_chunk_pos(ChunkPos::from_world_pos(Vec3::new(wcx, wcy, iz)))
            {
                if chunk.is_initialized() {
                    let b = chunk.get_block(
                        (wcx - chunk.pos.x as f32 * CHUNK_SIZE as f32).floor() as usize,
                        (wcy - chunk.pos.y as f32 * CHUNK_SIZE as f32).floor() as usize,
                        (iz - chunk.pos.z as f32 * CHUNK_SIZE as f32).floor() as usize,
                    );
                    if b.is_solid() || (hit_water && b.kind() == Block::water_block()) {
                        block = Some(b);
                    }
                }
            }
            if let Some(hit_block) = block {
                // The ray hit a block
                return Some(RayHit {
                    // Position of ray hit
                    hit_pos: Vec3::new(
                        position.x + t * direction.x,
                        position.y + t * direction.y,
                        position.z + t * direction.z,
                    ),
                    // Block coordinate of block that was hit
                    hit_block_pos: Vec3::new(wcx, wcy, iz),
                    // Normal of ray hit (assumes cubical blocks)
                    hit_norm: Vec3::new(
                        if stepped_index == 0 {
                            -stepx as f32
                        } else {
                            0.0
                        },
                        if stepped_index == 1 {
                            -stepy as f32
                        } else {
                            0.0
                        },
                        if stepped_index == 2 {
                            -stepz as f32
                        } else {
                            0.0
                        },
                    ),
                    hit_block,
                });
            }

            // On to the next block boundary
            if tx_max < ty_max {
                if tx_max < tz_max {
                    wcx = wcx + stepx as f32;
                    t = tx_max;
                    tx_max = tx_max + tx_delta;
                    stepped_index = 0;
                } else {
                    iz = iz + stepz as f32;
                    t = tz_max;
                    tz_max = tz_max + tz_delta;
                    stepped_index = 2;
                }
            } else {
                if ty_max < tz_max {
                    wcy = wcy + stepy as f32;
                    t = ty_max;
                    ty_max = ty_max + ty_delta;
                    stepped_index = 1;
                } else {
                    iz = iz + stepz as f32;
                    t = tz_max;
                    tz_max = tz_max + tz_delta;
                    stepped_index = 2;
                }
            }
        }
        // End of ray, no hit found
        None
    }

    /// Return collissions on all sides of an axis aligned box
    pub fn box_collisions(
        &self,
        p1: Vec3,
        p2: Vec3,
        velocity: Vec3,
    ) -> HashMap<Direction, Collision> {
        assert!(p1.x < p2.x);
        assert!(p1.y < p2.y);
        assert!(p1.z < p2.z);
        let mut collisions = Vec::new();
        // Check along the direction of the velocity for collisions
        let velocity_dir = velocity.normalize();
        // Check using a ray cast with maximum length of velocity
        let velocity_len = velocity.norm();
        // Check X
        if velocity.x.abs() > 0.0 {
            // Cast rays from direction we're moving in
            let mut p = p1.clone();
            if velocity.x > 0.0 {
                // Check from xp face
                p.x = p2.x;
            }
            while p.y <= p2.y {
                p.z = p1.z;
                while p.z <= p2.z {
                    if let Some(hit) = self.cast_ray(&p, &velocity_dir, velocity_len, false) {
                        collisions.push(Collision {
                            hit_pos: hit.hit_pos,
                            hit_delta: hit.hit_pos - p,
                            hit_norm: hit.hit_norm,
                            hit_block: hit.hit_block,
                        });
                    }
                    // Step to next block but make sure we check the corner as well
                    if p.z == p2.z {
                        break;
                    }
                    p.z = p.z + 1.0;
                    if p.z > p2.z {
                        p.z = p2.z;
                    }
                }
                // Step to next block but make sure we check the corner as well
                if p.y == p2.y {
                    break;
                }
                p.y = p.y + 1.0;
                if p.y > p2.y {
                    p.y = p2.y;
                }
            }
        }
        // Check Y
        if velocity.y.abs() > 0.0 {
            // Cast rays from direction we're moving in
            let mut p = p1.clone();
            if velocity.y > 0.0 {
                // Check from yp face
                p.y = p2.y;
            }
            while p.x <= p2.x {
                p.z = p1.z;
                while p.z <= p2.z {
                    if let Some(hit) = self.cast_ray(&p, &velocity_dir, velocity_len, false) {
                        collisions.push(Collision {
                            hit_pos: hit.hit_pos,
                            hit_delta: hit.hit_pos - p,
                            hit_norm: hit.hit_norm,
                            hit_block: hit.hit_block,
                        });
                    }
                    // Step to next block but make sure we check the corner as well
                    if p.z == p2.z {
                        break;
                    }
                    p.z = p.z + 1.0;
                    if p.z > p2.z {
                        p.z = p2.z;
                    }
                }
                // Step to next block but make sure we check the corner as well
                if p.x == p2.x {
                    break;
                }
                p.x = p.x + 1.0;
                if p.x > p2.x {
                    p.x = p2.x;
                }
            }
        }
        // Check Z
        if velocity.z.abs() > 0.0 {
            // Cast rays from direction we're moving in
            let mut p = p1.clone();
            if velocity.z > 0.0 {
                // Check from zp face
                p.z = p2.z;
            }
            while p.x <= p2.x {
                p.y = p1.y;
                while p.y <= p2.y {
                    if let Some(hit) = self.cast_ray(&p, &velocity_dir, velocity_len, false) {
                        collisions.push(Collision {
                            hit_pos: hit.hit_pos,
                            hit_delta: hit.hit_pos - p,
                            hit_norm: hit.hit_norm,
                            hit_block: hit.hit_block,
                        });
                    }
                    // Step to next block but make sure we check the corner as well
                    if p.y == p2.y {
                        break;
                    }
                    p.y = p.y + 1.0;
                    if p.y > p2.y {
                        p.y = p2.y;
                    }
                }
                // Step to next block but make sure we check the corner as well
                if p.x == p2.x {
                    break;
                }
                p.x = p.x + 1.0;
                if p.x > p2.x {
                    p.x = p2.x;
                }
            }
        }
        let mut result = HashMap::new();
        for collision in &collisions {
            let dir = Direction::from_block_normal(collision.hit_norm);
            let closest = result.entry(dir).or_insert(collision.clone());
            if collision.hit_delta.norm() < closest.hit_delta.norm() {
                result.insert(dir, collision.clone());
            }
        }
        result
    }

    /// Set a block in the buffer and add any chunks that need meshing to the dirty chunks set
    pub fn set_block_add_dirty(
        &mut self,
        wbx: i16,
        wby: i16,
        wbz: i16,
        block: Block,
        dirty_chunks: &mut HashSet<ChunkPos>,
    ) {
        // Get and check the chunk and block
        let chunk_pos = ChunkPos::from_world_block_coords(wbx, wby, wbz);
        let chunk_opt = self.chunks.get_mut_chunk_pos(chunk_pos);
        if chunk_opt == None {
            warn!(
                "attempted set block {},{},{} on chunk that is not in buffer",
                wbx, wby, wbz
            );
            return;
        }
        let chunk = chunk_opt.unwrap();
        if !chunk.is_initialized() {
            warn!(
                "attempted set block {},{},{} on uninitialized chunk",
                wbx, wby, wbz
            );
            return;
        }
        let old_block = chunk.get_block_world(wbx, wby, wbz);
        let mut new_block = block;
        if old_block.kind() == new_block.kind() {
            // No need to do anything if the block does not change
            return;
        }
        let new_block_light = self.block_registry.get(new_block.kind()).light;
        if new_block_light > 0 {
            new_block.set_light(new_block_light);
        }
        // Now set the block
        if let Some(chunk) = self.chunks.get_mut_chunk_pos(chunk_pos) {
            if chunk.is_initialized() {
                let x_rel = (wbx - chunk_pos.x * CHUNK_SIZE as i16) as usize;
                let y_rel = (wby - chunk_pos.y * CHUNK_SIZE as i16) as usize;
                let z_rel = (wbz - chunk_pos.z * CHUNK_SIZE as i16) as usize;
                // If a light emitting block is removed, the propagated light should also be removed.
                let old_block_light = self.block_registry.get(old_block.kind()).light;
                if old_block_light > 0 || old_block.get_light() > 0 {
                    self.chunks.remove_block_and_light(
                        wbx,
                        wby,
                        wbz,
                        old_block.get_light(),
                        new_block,
                        dirty_chunks,
                    );
                } else {
                    chunk.set_block(x_rel, y_rel, z_rel, new_block);
                }
                if new_block.get_light() > 0 {
                    // Propagate light from this block outward
                    self.chunks.propagate_light(wbx, wby, wbz, dirty_chunks);
                } else if new_block.is_transparent() {
                    // Propagate light from other sources after this light block was removed
                    self.chunks
                        .propagate_light_after_removal(wbx, wby, wbz, dirty_chunks);
                }
                // Determine which chunks need remeshing
                dirty_chunks.insert(chunk_pos.clone());
                if x_rel == 0 {
                    dirty_chunks.insert(ChunkPos::new(chunk_pos.x - 1, chunk_pos.y, chunk_pos.z));
                } else if x_rel == CHUNK_SIZE - 1 {
                    dirty_chunks.insert(ChunkPos::new(chunk_pos.x + 1, chunk_pos.y, chunk_pos.z));
                }
                if y_rel == 0 {
                    dirty_chunks.insert(ChunkPos::new(chunk_pos.x, chunk_pos.y - 1, chunk_pos.z));
                } else if y_rel == CHUNK_SIZE - 1 {
                    dirty_chunks.insert(ChunkPos::new(chunk_pos.x, chunk_pos.y + 1, chunk_pos.z));
                }
                if z_rel == 0 {
                    dirty_chunks.insert(ChunkPos::new(chunk_pos.x, chunk_pos.y, chunk_pos.z - 1));
                } else if z_rel == CHUNK_SIZE - 1 {
                    dirty_chunks.insert(ChunkPos::new(chunk_pos.x, chunk_pos.y, chunk_pos.z + 1));
                }
            }
        }
    }

    pub fn store_column(&mut self, column: ChunkColumn) {
        self.chunks.store_column(column);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Collision {
    pub hit_pos: Vec3,
    pub hit_delta: Vec3,
    pub hit_norm: Vec3,
    pub hit_block: Block,
}

#[derive(Debug, Copy, Clone)]
pub struct RayHit {
    pub hit_block_pos: Vec3,
    pub hit_pos: Vec3,
    pub hit_norm: Vec3,
    pub hit_block: Block,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
    XP,
    XM,
    YP,
    YM,
    ZP,
    ZM,
    NONE,
}

impl Direction {
    pub fn from_block_normal(vector: Vec3) -> Direction {
        if vector.x > 0.0 {
            Direction::XM
        } else if vector.x < 0.0 {
            Direction::XP
        } else if vector.y > 0.0 {
            Direction::YM
        } else if vector.y < 0.0 {
            Direction::YP
        } else if vector.z > 0.0 {
            Direction::ZM
        } else if vector.z < 0.0 {
            Direction::ZP
        } else {
            Direction::NONE
        }
    }
}

pub fn is_column_inside_frustrum_xy(
    frustum_checker: &FrustumChecker,
    col: &ChunkColumnPos,
) -> bool {
    let cz = CHUNK_SIZE as f32;
    let p = Vec3::new(
        col.x as f32 * CHUNK_SIZE as f32,
        col.y as f32 * CHUNK_SIZE as f32,
        0.0,
    );
    if frustum_checker.is_inside_frustrum_xy(p) {
        return true;
    }
    if frustum_checker.is_inside_frustrum_xy(p + Vec3::new(cz, 0.0, 0.0)) {
        return true;
    }
    if frustum_checker.is_inside_frustrum_xy(p + Vec3::new(0.0, cz, 0.0)) {
        return true;
    }
    if frustum_checker.is_inside_frustrum_xy(p + Vec3::new(cz, cz, 0.0)) {
        return true;
    }
    false
}
