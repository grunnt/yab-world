pub mod comms;
pub mod messages;
pub mod read_from;
pub mod rle;
pub mod write_to;

pub use comms::CommChannel;
pub use comms::CommsClient;
pub use comms::CommsError;
pub use comms::CommsServer;
pub use messages::ClientMessage;
pub use messages::ServerMessage;
pub use rle::RleDecode;
pub use rle::RleEncode;

use std::io::{Read, Write};

pub const DEFAULT_TCP_PORT: u32 = 34254;

pub trait SerializeMessage<T> {
    fn serialize_into_writer<W: Write>(&self, writer: &mut W) -> Result<(), CommsError>;
    fn deserialize_from_reader<R: Read>(reader: &mut R) -> Result<T, CommsError>;
}
