use crate::comms::CommsError;
use std::io::Read;

pub trait ReadFrom<R>: Sized
where
    R: Read,
{
    fn read_from(reader: &mut R) -> Result<Self, CommsError>;
}

impl<R> ReadFrom<R> for u8
where
    R: Read,
{
    fn read_from(reader: &mut R) -> Result<Self, CommsError> {
        let mut value: [u8; 1] = [0; 1];
        reader.read_exact(&mut value)?;
        Ok(value[0])
    }
}

impl<R> ReadFrom<R> for u16
where
    R: Read,
{
    fn read_from(reader: &mut R) -> Result<Self, CommsError> {
        let mut bytes: [u8; 2] = [0; 2];
        reader.read_exact(&mut bytes)?;
        Ok(u16::from_le_bytes(bytes))
    }
}

impl<R> ReadFrom<R> for i16
where
    R: Read,
{
    fn read_from(reader: &mut R) -> Result<Self, CommsError> {
        let mut bytes: [u8; 2] = [0; 2];
        reader.read_exact(&mut bytes)?;
        Ok(i16::from_le_bytes(bytes))
    }
}

impl<R> ReadFrom<R> for u32
where
    R: Read,
{
    fn read_from(reader: &mut R) -> Result<Self, CommsError> {
        let mut bytes: [u8; 4] = [0; 4];
        reader.read_exact(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }
}

impl<R> ReadFrom<R> for f32
where
    R: Read,
{
    fn read_from(reader: &mut R) -> Result<Self, CommsError> {
        let mut bytes: [u8; 4] = [0; 4];
        reader.read_exact(&mut bytes)?;
        Ok(f32::from_le_bytes(bytes))
    }
}

impl<R> ReadFrom<R> for String
where
    R: Read,
{
    fn read_from(reader: &mut R) -> Result<Self, CommsError> {
        let size = u16::read_from(reader)?;
        let mut bytes = vec![0; size as usize];
        reader.read_exact(&mut bytes)?;
        Ok(String::from_utf8(bytes).unwrap())
    }
}
