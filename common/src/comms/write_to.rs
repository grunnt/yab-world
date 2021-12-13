use crate::comms::CommsError;
use std::io::Write;

pub trait WriteTo<W>
where
    W: Write,
{
    fn write_to(&self, writer: &mut W) -> Result<(), CommsError>;
}

impl<W> WriteTo<W> for u8
where
    W: Write,
{
    fn write_to(&self, writer: &mut W) -> Result<(), CommsError> {
        Ok(writer.write_all(&[*self])?)
    }
}

impl<W> WriteTo<W> for u16
where
    W: Write,
{
    fn write_to(&self, writer: &mut W) -> Result<(), CommsError> {
        Ok(writer.write_all(&self.to_le_bytes())?)
    }
}

impl<W> WriteTo<W> for i16
where
    W: Write,
{
    fn write_to(&self, writer: &mut W) -> Result<(), CommsError> {
        Ok(writer.write_all(&self.to_le_bytes())?)
    }
}

impl<W> WriteTo<W> for u32
where
    W: Write,
{
    fn write_to(&self, writer: &mut W) -> Result<(), CommsError> {
        Ok(writer.write_all(&self.to_le_bytes())?)
    }
}

impl<W> WriteTo<W> for f32
where
    W: Write,
{
    fn write_to(&self, writer: &mut W) -> Result<(), CommsError> {
        Ok(writer.write_all(&self.to_le_bytes())?)
    }
}

impl<W> WriteTo<W> for String
where
    W: Write,
{
    fn write_to(&self, writer: &mut W) -> Result<(), CommsError> {
        let bytes = self.as_bytes();
        assert!(bytes.len() < std::u16::MAX as usize);
        (bytes.len() as u16).write_to(writer)?;
        Ok(writer.write_all(&bytes)?)
    }
}
