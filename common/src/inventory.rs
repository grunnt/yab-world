use crate::{
    block::Block,
    comms::{read_from::ReadFrom, write_to::WriteTo, CommsError},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Read, Write},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Inventory {
    blocks: HashMap<Block, u32>,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory {
            blocks: HashMap::new(),
        }
    }

    pub fn add(&mut self, block: Block, count: u32) {
        *self.blocks.entry(block).or_insert(0) += count;
    }

    pub fn remove(&mut self, block: Block, count: u32) {
        *self.blocks.entry(block).or_insert(0) -= count;
    }

    pub fn count(&self, block: Block) -> u32 {
        if let Some(count) = self.blocks.get(&block) {
            *count
        } else {
            0
        }
    }

    pub fn blocks(&self) -> &HashMap<Block, u32> {
        &self.blocks
    }
}

impl<W> WriteTo<W> for Inventory
where
    W: Write,
{
    fn write_to(&self, writer: &mut W) -> Result<(), CommsError> {
        let count = self.blocks.len() as u32;
        count.write_to(writer)?;
        for (block_type, block_count) in &self.blocks {
            block_type.write_to(writer)?;
            block_count.write_to(writer)?;
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
        let count = u32::read_from(reader)?;
        for _ in 0..count {
            let block = Block::read_from(reader)?;
            let count = u32::read_from(reader)?;
            inventory.add(block, count);
        }
        Ok(inventory)
    }
}
