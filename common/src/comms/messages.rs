use crate::block::*;
use crate::comms::read_from::ReadFrom;
use crate::comms::write_to::WriteTo;
use crate::comms::*;
use crate::{chunk::*, inventory::Inventory};
use log::*;
use std::collections::HashSet;
use std::io::{Read, Write};

#[derive(Debug, PartialEq, Clone)]
pub enum ClientMessage {
    SignIn {
        username: String,
    },
    PositionUpdate {
        x: f32,
        y: f32,
        z: f32,
        yaw: f32,
        pitch: f32,
    },
    Subscribe {
        columns: Vec<ChunkColumnPos>,
    },
    Unsubscribe {
        columns: HashSet<ChunkColumnPos>,
    },
    SetBlock {
        wbx: i16,
        wby: i16,
        wbz: i16,
        block: Block,
    },
    // Message {
    //     text: String,
    // },
    SignOut {},
}

const CM_VARIANT_SIGN_IN: u8 = 0;
const CM_VARIANT_SIGN_OUT: u8 = 1;
const CM_VARIANT_POSITION_UPDATE: u8 = 2;
const CM_VARIANT_SUBSCRIBE: u8 = 3;
const CM_VARIANT_UNSUBSCRIBE: u8 = 4;
const CM_VARIANT_SET_BLOCK: u8 = 5;

/// Manual serialization because serde / bincode is bugged and to use RLE
/// It's ugly but it works
impl SerializeMessage<ClientMessage> for ClientMessage {
    fn serialize_into_writer<W: Write>(&self, writer: &mut W) -> Result<(), CommsError> {
        match self {
            ClientMessage::SignIn { username } => {
                CM_VARIANT_SIGN_IN.write_to(writer)?;
                username.write_to(writer)?;
            }
            ClientMessage::SignOut {} => {
                CM_VARIANT_SIGN_OUT.write_to(writer)?;
            }
            ClientMessage::PositionUpdate {
                x,
                y,
                z,
                yaw,
                pitch,
            } => {
                CM_VARIANT_POSITION_UPDATE.write_to(writer)?;
                x.write_to(writer)?;
                y.write_to(writer)?;
                z.write_to(writer)?;
                yaw.write_to(writer)?;
                pitch.write_to(writer)?;
            }
            ClientMessage::Subscribe { columns } => {
                CM_VARIANT_SUBSCRIBE.write_to(writer)?;
                assert!(columns.len() < std::u16::MAX as usize);
                (columns.len() as u16).write_to(writer)?;
                for cp in columns {
                    cp.x.write_to(writer)?;
                    cp.y.write_to(writer)?;
                }
            }
            ClientMessage::Unsubscribe { columns } => {
                CM_VARIANT_UNSUBSCRIBE.write_to(writer)?;
                assert!(columns.len() < std::u16::MAX as usize);
                (columns.len() as u16).write_to(writer)?;
                for cp in columns {
                    cp.x.write_to(writer)?;
                    cp.y.write_to(writer)?;
                }
            }
            ClientMessage::SetBlock {
                wbx,
                wby,
                wbz,
                block,
            } => {
                CM_VARIANT_SET_BLOCK.write_to(writer)?;
                wbx.write_to(writer)?;
                wby.write_to(writer)?;
                wbz.write_to(writer)?;
                block.write_to(writer)?;
            }
        }
        Ok(())
    }

    fn deserialize_from_reader<R: Read>(reader: &mut R) -> Result<ClientMessage, CommsError> {
        let enum_variant = u8::read_from(reader)?;
        match enum_variant {
            CM_VARIANT_SIGN_IN => {
                let username = String::read_from(reader)?;
                let message = ClientMessage::SignIn { username };
                Ok(message)
            }
            CM_VARIANT_SIGN_OUT => Ok(ClientMessage::SignOut {}),
            CM_VARIANT_POSITION_UPDATE => {
                let x = f32::read_from(reader)?;
                let y = f32::read_from(reader)?;
                let z = f32::read_from(reader)?;
                let yaw = f32::read_from(reader)?;
                let pitch = f32::read_from(reader)?;
                let message = ClientMessage::PositionUpdate {
                    x,
                    y,
                    z,
                    yaw,
                    pitch,
                };
                Ok(message)
            }
            CM_VARIANT_SUBSCRIBE => {
                let size = u16::read_from(reader)? as usize;
                let mut columns = Vec::new();
                for _ in 0..size {
                    let x = i16::read_from(reader)?;
                    let y = i16::read_from(reader)?;
                    columns.push(ChunkColumnPos { x, y });
                }
                assert!(size == columns.len());
                Ok(ClientMessage::Subscribe { columns })
            }
            CM_VARIANT_UNSUBSCRIBE => {
                let size = u16::read_from(reader)? as usize;
                let mut columns = HashSet::new();
                for _ in 0..size {
                    let x = i16::read_from(reader)?;
                    let y = i16::read_from(reader)?;
                    columns.insert(ChunkColumnPos { x, y });
                }
                assert!(size == columns.len());
                Ok(ClientMessage::Unsubscribe { columns })
            }
            CM_VARIANT_SET_BLOCK => {
                let wbx = i16::read_from(reader)?;
                let wby = i16::read_from(reader)?;
                let wbz = i16::read_from(reader)?;
                let block = u16::read_from(reader)?;
                let message = ClientMessage::SetBlock {
                    wbx,
                    wby,
                    wbz,
                    block,
                };
                Ok(message)
            }
            _ => {
                error!("Unknown enum variant {}", enum_variant);
                Err(CommsError::ProtocolError)
            }
        }
    }
}

#[cfg(test)]
mod serialize_client_messages {

    use crate::chunk::*;
    use crate::comms::*;
    use std::collections::HashSet;

    #[test]
    fn sign_in() {
        test(ClientMessage::SignIn {
            username: "my user".to_string(),
        });
    }

    #[test]
    fn sign_out() {
        test(ClientMessage::SignOut {});
    }

    #[test]
    fn position_update() {
        test(ClientMessage::PositionUpdate {
            x: 123.0,
            y: 456.0,
            z: 789.0,
            yaw: 1.23,
            pitch: 5.67,
        });
    }

    #[test]
    fn subscribe() {
        let mut columns = Vec::new();
        for i in 0..100 {
            columns.push(ChunkColumnPos {
                x: -5 + i,
                y: 5 - i,
            });
        }
        test(ClientMessage::Subscribe { columns });
    }

    #[test]
    fn unsubscribe() {
        let mut columns = HashSet::new();
        for i in 0..100 {
            columns.insert(ChunkColumnPos {
                x: -5 + i,
                y: 5 - i,
            });
        }
        test(ClientMessage::Unsubscribe { columns });
    }

    #[test]
    fn set_block() {
        test(ClientMessage::SetBlock {
            wbx: 123,
            wby: -456,
            wbz: 5,
            block: 9,
        });
    }

    // Serialize and deserialize a message and compare it ith the original
    fn test(message: ClientMessage) {
        let mut buf: Vec<u8> = Vec::new();
        message.clone().serialize_into_writer(&mut buf).unwrap();
        let message_out = ClientMessage::deserialize_from_reader(&mut buf.as_slice()).unwrap();
        assert_eq!(message, message_out);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ServerMessage {
    SignInConfirm {
        player_id: u8,
        x: f32,
        y: f32,
        z: f32,
        yaw: f32,
        pitch: f32,
        inventory: Inventory,
        gametime: f32,
        block_registry: String,
        resource_registry: String,
    },
    ChunkColumn {
        col: ChunkColumnPos,
        block_data: Vec<Vec<u8>>,
    },
    // Message {
    //     player_id: u8,
    //     text: String,
    // },
    SetBlock {
        wbx: i16,
        wby: i16,
        wbz: i16,
        block: Block,
    },
    PlayerSpawn {
        x: f32,
        y: f32,
        z: f32,
        yaw: f32,
        pitch: f32,
        player_id: u8,
        username: String,
    },
    PositionUpdate {
        player_id: u8,
        x: f32,
        y: f32,
        z: f32,
        yaw: f32,
        pitch: f32,
    },
    PlayerDespawn {
        player_id: u8,
    },
    // ClientDisconnect {},
}

const SM_VARIANT_CHUNK_COLUMN: u8 = 0;
const SM_VARIANT_SET_BLOCK: u8 = 1;
const SM_VARIANT_PLAYER_SPAWN: u8 = 2;
const SM_VARIANT_PLAYER_DESPAWN: u8 = 3;
const SM_VARIANT_POSITION_UPDATE: u8 = 4;
const SM_VARIANT_SIGN_IN_CONFIRM: u8 = 5;

/// Manual serialization because serde / bincode is bugged and to use RLE
/// It's ugly but it works
impl SerializeMessage<ServerMessage> for ServerMessage {
    fn serialize_into_writer<W: Write>(&self, writer: &mut W) -> Result<(), CommsError> {
        match self {
            ServerMessage::SignInConfirm {
                player_id,
                x,
                y,
                z,
                yaw,
                pitch,
                inventory,
                gametime,
                block_registry,
                resource_registry,
            } => {
                SM_VARIANT_SIGN_IN_CONFIRM.write_to(writer)?;
                player_id.write_to(writer)?;
                x.write_to(writer)?;
                y.write_to(writer)?;
                z.write_to(writer)?;
                yaw.write_to(writer)?;
                pitch.write_to(writer)?;
                inventory.write_to(writer)?;
                gametime.write_to(writer)?;
                block_registry.write_to(writer)?;
                resource_registry.write_to(writer)?;
            }
            ServerMessage::ChunkColumn { col, block_data } => {
                SM_VARIANT_CHUNK_COLUMN.write_to(writer)?;
                col.x.write_to(writer)?;
                col.y.write_to(writer)?;
                for z in 0..WORLD_HEIGHT_CHUNKS {
                    let blocks = &block_data[z];
                    assert!(blocks.len() < std::u16::MAX as usize);
                    (blocks.len() as u16).write_to(writer)?;
                    if let Err(_) = writer.write_all(&blocks) {
                        return Err(CommsError::Disconnected);
                    };
                }
            }
            ServerMessage::SetBlock {
                wbx,
                wby,
                wbz,
                block,
            } => {
                SM_VARIANT_SET_BLOCK.write_to(writer)?;
                wbx.write_to(writer)?;
                wby.write_to(writer)?;
                wbz.write_to(writer)?;
                block.write_to(writer)?;
            }
            ServerMessage::PlayerSpawn {
                x,
                y,
                z,
                yaw,
                pitch,
                player_id,
                username,
            } => {
                SM_VARIANT_PLAYER_SPAWN.write_to(writer)?;
                x.write_to(writer)?;
                y.write_to(writer)?;
                z.write_to(writer)?;
                yaw.write_to(writer)?;
                pitch.write_to(writer)?;
                player_id.write_to(writer)?;
                username.write_to(writer)?;
            }
            ServerMessage::PlayerDespawn { player_id } => {
                SM_VARIANT_PLAYER_DESPAWN.write_to(writer)?;
                player_id.write_to(writer)?;
            }
            ServerMessage::PositionUpdate {
                x,
                y,
                z,
                yaw,
                pitch,
                player_id,
            } => {
                SM_VARIANT_POSITION_UPDATE.write_to(writer)?;
                x.write_to(writer)?;
                y.write_to(writer)?;
                z.write_to(writer)?;
                yaw.write_to(writer)?;
                pitch.write_to(writer)?;
                player_id.write_to(writer)?;
            }
        }
        Ok({})
    }

    fn deserialize_from_reader<R: Read>(reader: &mut R) -> Result<ServerMessage, CommsError> {
        let enum_variant = u8::read_from(reader)?;
        match enum_variant {
            SM_VARIANT_SIGN_IN_CONFIRM => {
                let player_id = u8::read_from(reader)?;
                let x = f32::read_from(reader)?;
                let y = f32::read_from(reader)?;
                let z = f32::read_from(reader)?;
                let yaw = f32::read_from(reader)?;
                let pitch = f32::read_from(reader)?;
                let inventory = Inventory::read_from(reader)?;
                let gametime = f32::read_from(reader)?;
                let block_registry = String::read_from(reader)?;
                let resource_registry = String::read_from(reader)?;
                Ok(ServerMessage::SignInConfirm {
                    player_id,
                    x,
                    y,
                    z,
                    yaw,
                    pitch,
                    inventory,
                    gametime,
                    block_registry,
                    resource_registry,
                })
            }
            SM_VARIANT_CHUNK_COLUMN => {
                let x = i16::read_from(reader)?;
                let y = i16::read_from(reader)?;
                let col = ChunkColumnPos { x, y };
                let mut block_data = Vec::new();
                for _ in 0..WORLD_HEIGHT_CHUNKS {
                    let length = u16::read_from(reader)? as usize;
                    let mut blocks = vec![0; length];
                    if let Err(_) = reader.read_exact(&mut blocks) {
                        return Err(CommsError::Disconnected);
                    }
                    block_data.push(blocks);
                }
                Ok(ServerMessage::ChunkColumn { col, block_data })
            }
            SM_VARIANT_SET_BLOCK => {
                let wbx = i16::read_from(reader)?;
                let wby = i16::read_from(reader)?;
                let wbz = i16::read_from(reader)?;
                let block = u16::read_from(reader)?;

                let message = ServerMessage::SetBlock {
                    wbx,
                    wby,
                    wbz,
                    block,
                };
                Ok(message)
            }
            SM_VARIANT_PLAYER_SPAWN => {
                let x = f32::read_from(reader)?;
                let y = f32::read_from(reader)?;
                let z = f32::read_from(reader)?;
                let yaw = f32::read_from(reader)?;
                let pitch = f32::read_from(reader)?;
                let player_id = u8::read_from(reader)?;
                let username = String::read_from(reader)?;

                let message = ServerMessage::PlayerSpawn {
                    x,
                    y,
                    z,
                    yaw,
                    pitch,
                    player_id,
                    username,
                };
                Ok(message)
            }
            SM_VARIANT_PLAYER_DESPAWN => {
                let player_id = u8::read_from(reader)?;
                let message = ServerMessage::PlayerDespawn { player_id };
                Ok(message)
            }
            SM_VARIANT_POSITION_UPDATE => {
                let x = f32::read_from(reader)?;
                let y = f32::read_from(reader)?;
                let z = f32::read_from(reader)?;
                let yaw = f32::read_from(reader)?;
                let pitch = f32::read_from(reader)?;
                let player_id = u8::read_from(reader)?;

                let message = ServerMessage::PositionUpdate {
                    x,
                    y,
                    z,
                    yaw,
                    pitch,
                    player_id,
                };
                Ok(message)
            }
            _ => {
                error!("Unknown enum variant {}", enum_variant);
                Err(CommsError::ProtocolError)
            }
        }
    }
}

#[cfg(test)]
mod serialize_server_messages {

    use crate::{block::BlockRegistry, comms::*, resource::ResourceRegistry};
    use crate::{chunk::*, inventory::Inventory};

    #[test]
    fn sign_in_confirm() {
        let mut inventory = Inventory::new();
        inventory.add(0, 123);
        inventory.add(1, 54);
        inventory.add(4, 99);
        test(ServerMessage::SignInConfirm {
            player_id: 3,
            x: 123.0,
            y: 456.0,
            z: -891.0,
            yaw: 1.23,
            pitch: -1.234,
            inventory,
            gametime: 1.23,
            block_registry: serde_json::to_string(&BlockRegistry::default()).unwrap(),
            resource_registry: serde_json::to_string(&ResourceRegistry::default()).unwrap(),
        });
    }

    #[test]
    fn chunk() {
        let mut block_data = Vec::new();
        for _ in 0..WORLD_HEIGHT_CHUNKS {
            let mut blocks = Vec::new();
            blocks.push(1);
            block_data.push(blocks);
        }
        test(ServerMessage::ChunkColumn {
            col: ChunkColumnPos::new(1, 2),
            block_data,
        });
    }

    #[test]
    fn set_block() {
        test(ServerMessage::SetBlock {
            wbx: 123,
            wby: -456,
            wbz: 5,
            block: 9,
        });
    }

    #[test]
    fn player_spawn() {
        test(ServerMessage::PlayerSpawn {
            x: 123.0,
            y: 456.0,
            z: -789.0,
            yaw: 1.23,
            pitch: 3.434,
            player_id: 5,
            username: String::from("other user"),
        });
    }

    #[test]
    fn player_despawn() {
        test(ServerMessage::PlayerDespawn { player_id: 123 });
    }

    #[test]
    fn position_update() {
        test(ServerMessage::PositionUpdate {
            x: 123.0,
            y: 456.0,
            z: -789.0,
            yaw: 1.434,
            pitch: 4.552,
            player_id: 5,
        });
    }

    // Serialize and deserialize a message and compare it ith the original
    fn test(message: ServerMessage) {
        let mut buf: Vec<u8> = Vec::new();
        message.clone().serialize_into_writer(&mut buf).unwrap();
        let message_out = ServerMessage::deserialize_from_reader(&mut buf.as_slice()).unwrap();
        assert_eq!(message, message_out);
    }
}
