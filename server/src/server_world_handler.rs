use std::collections::HashMap;

use common::chunk::ChunkColumnPos;

type MarkerHandle = u32;

#[derive(Clone)]
pub struct GeneratorMarker {
    pub handle: MarkerHandle,
    pub position: ChunkColumnPos,
    pub chunk_range: usize,
}
pub struct ServerWorldHandler {
    next_maker_handle: MarkerHandle,
    markers: HashMap<MarkerHandle, GeneratorMarker>,
    // TODO add world store here
    // TODO add chunk buffer here
}

impl ServerWorldHandler {
    pub fn new() -> Self {
        // TODO spin up generator threads
        ServerWorldHandler {
            next_maker_handle: 1,
            markers: HashMap::new(),
        }
    }

    pub fn add_marker(&mut self, position: ChunkColumnPos, chunk_range: usize) -> MarkerHandle {
        let handle = self.next_maker_handle;
        let marker = GeneratorMarker {
            handle,
            position,
            chunk_range,
        };
        self.next_maker_handle += 1;
        self.markers.insert(marker.handle, marker);
        handle
    }

    pub fn update_marker(
        &mut self,
        handle: MarkerHandle,
        position: ChunkColumnPos,
        chunk_range: usize,
    ) {
        if let Some(marker) = self.markers.get_mut(&handle) {
            marker.position = position;
            marker.chunk_range = chunk_range;
        }
    }

    pub fn delete_marker(&mut self, handle: MarkerHandle) {
        self.markers.remove(&handle);
    }
}
