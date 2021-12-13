use crate::{
    comms::{read_from::ReadFrom, write_to::WriteTo, CommsError},
    resource::Resource,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Read, Write},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Inventory {
    resources: HashMap<Resource, u32>,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory {
            resources: HashMap::new(),
        }
    }

    pub fn add(&mut self, resource: Resource, count: u32) {
        *self.resources.entry(resource).or_insert(0) += count;
    }

    pub fn remove(&mut self, resource: Resource, count: u32) {
        *self.resources.entry(resource).or_insert(0) -= count;
    }

    pub fn count(&self, resource: Resource) -> u32 {
        if let Some(count) = self.resources.get(&resource) {
            *count
        } else {
            0
        }
    }

    pub fn resources(&self) -> &HashMap<Resource, u32> {
        &self.resources
    }
}

impl<W> WriteTo<W> for Inventory
where
    W: Write,
{
    fn write_to(&self, writer: &mut W) -> Result<(), CommsError> {
        let count = self.resources.len() as u16;
        count.write_to(writer)?;
        for (resource_type, resource_count) in &self.resources {
            resource_type.write_to(writer)?;
            resource_count.write_to(writer)?;
        }
        Ok(())
    }
}

impl<R> ReadFrom<R> for Inventory
where
    R: Read,
{
    fn read_from(reader: &mut R) -> Result<Self, CommsError> {
        let mut inventory = Inventory::new();
        let count = u16::read_from(reader)?;
        for _ in 0..count {
            let resource = Resource::read_from(reader)?;
            let count = u32::read_from(reader)?;
            inventory.add(resource, count);
        }
        Ok(inventory)
    }
}
