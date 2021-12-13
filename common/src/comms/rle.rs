use crate::comms::read_from::ReadFrom;
use crate::comms::write_to::WriteTo;
use crate::comms::CommsError;
use std::io::{Read, Write};

pub trait RleEncode<T, W>
where
    T: WriteTo<W> + PartialEq,
    W: Write,
{
    /// Write a vector of values to a writer using run-length encoding
    fn rle_encode_to(&self, writer: &mut W) -> Result<(), CommsError>;
}

impl<T, W> RleEncode<T, W> for Vec<T>
where
    T: WriteTo<W> + PartialEq,
    W: Write,
{
    fn rle_encode_to(&self, writer: &mut W) -> Result<(), CommsError> {
        assert!(self.len() < std::u16::MAX as usize);
        assert!(!self.is_empty());
        (self.len() as u16).write_to(writer)?;
        // If one block (i.e. a solid chunk) just write it, otherwise use RLE
        if self.len() == 1 {
            self[0].write_to(writer)?;
        } else {
            let mut run_block = &self[0];
            let mut count: u16 = 0;
            for block in self {
                if block == run_block {
                    // Continue run
                    count = count + 1;
                } else {
                    // New run
                    count.write_to(writer)?;
                    run_block.write_to(writer)?;
                    run_block = block;
                    count = 1;
                }
            }
            // Write last run
            count.write_to(writer)?;
            run_block.write_to(writer)?;
        }
        Ok(())
    }
}

pub trait RleDecode<T, R>: Sized
where
    T: ReadFrom<R> + PartialEq + std::fmt::Display,
    R: Read,
{
    /// Read a vector of values from a run-length encoded reader
    fn rle_decode_from(reader: &mut R) -> Result<Self, CommsError>;
}

impl<T, R> RleDecode<T, R> for Vec<T>
where
    T: ReadFrom<R> + PartialEq + Copy + std::fmt::Display,
    R: Read,
{
    fn rle_decode_from(reader: &mut R) -> Result<Self, CommsError> {
        let mut blocks = Vec::new();
        let block_count = u16::read_from(reader)? as usize;
        assert!(block_count > 0);
        // If one block (i.e. a solid chunk) just read it, otherwise use RLE
        if block_count == 1 {
            blocks.push(T::read_from(reader)?);
        } else {
            let mut count = 0;
            while count < block_count {
                let run_count = u16::read_from(reader)?;
                let run_block = T::read_from(reader)?;
                for _ in 0..run_count {
                    blocks.push(run_block);
                }
                count = count + run_count as usize;
            }
        }
        assert!(blocks.len() == block_count);
        Ok(blocks)
    }
}
