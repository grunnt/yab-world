use crate::comms::*;
use crossbeam::channel::*;
use crossbeam::unbounded;
use log::*;
use std::{fmt, sync::atomic::AtomicBool, thread::sleep, time};
use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
};
use std::{sync::atomic::Ordering, thread};

#[derive(Debug)]
pub enum CommsError {
    Disconnected,
    ProtocolError,
}

impl fmt::Display for CommsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommsError::Disconnected => write!(f, "Disconnected"),
            CommsError::ProtocolError => write!(f, "Protocol error"),
        }
    }
}

impl From<std::io::Error> for CommsError {
    fn from(_error: std::io::Error) -> Self {
        CommsError::Disconnected
    }
}

// Communications client: connect to server, read and send messages
pub struct CommsClient {
    stream: TcpStream,
    receive_rx: Receiver<ServerMessage>,
    col_receive_rx: Receiver<ServerMessage>,
}

impl CommsClient {
    pub fn new(server_address: SocketAddr) -> CommsClient {
        info!("Connecting to {}", server_address);
        let stream = TcpStream::connect(server_address).unwrap();
        let mut read_stream = stream.try_clone().unwrap();

        // Start message receiver thread
        let (receive_tx, receive_rx) = unbounded();
        let (col_receive_tx, col_receive_rx) = unbounded();
        thread::Builder::new()
            .name("client_receiver".to_string())
            .spawn(move || loop {
                match ServerMessage::deserialize_from_reader(&mut read_stream) {
                    Ok(message) => {
                        // The world worker thread receives the chunk columns directly to avoid hiccups in the main thread
                        let to_col_receiver = match message {
                            ServerMessage::ChunkColumn { .. } => true,
                            _ => false,
                        };
                        if to_col_receiver {
                            if col_receive_tx.send(message).is_err() {
                                panic!("Cannot send");
                            }
                        } else {
                            if receive_tx.send(message).is_err() {
                                panic!("Cannot send");
                            }
                        }
                    }
                    Err(e) => {
                        info!("Receive error: {}", e);
                        break;
                    }
                }
            })
            .unwrap();

        CommsClient {
            receive_rx,
            stream,
            col_receive_rx,
        }
    }

    pub fn send(&mut self, message: ClientMessage) -> Result<(), CommsError> {
        message.serialize_into_writer(&mut self.stream)?;
        Ok(())
    }

    pub fn try_receive(&mut self) -> Option<ServerMessage> {
        let message = self.receive_rx.try_recv();
        if message.is_ok() {
            Some(message.unwrap())
        } else {
            None
        }
    }

    pub fn clone_col_receiver(&self) -> Receiver<ServerMessage> {
        self.col_receive_rx.clone()
    }

    pub fn disconnect(&mut self) {
        debug!("Disconnecting comms client");
        if let Err(e) = self.stream.shutdown(std::net::Shutdown::Both) {
            warn!("Error disconnecting: {:?}", e);
        }
    }
}

// Communications server: accepts new connections, reads messages from clients
pub struct CommsServer {
    channel_rx: Receiver<CommChannel>,
    shutdown: Arc<AtomicBool>,
}

impl CommsServer {
    pub fn new(address: &str) -> CommsServer {
        let (channel_tx, channel_rx) = unbounded();
        let listener = TcpListener::bind(address).unwrap();
        listener
            .set_nonblocking(true)
            .expect("Cannot set non-blocking");
        info!("Listening at {}", address);
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_cloned = shutdown.clone();
        thread::Builder::new()
            .name("server_listener".to_string())
            .spawn(move || {
                let mut last_client_id = 0;
                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => {
                            let (msg_tx, msg_rx) = unbounded();
                            let client_id = last_client_id + 1;
                            last_client_id = client_id;
                            channel_tx
                                .send(CommChannel {
                                    client_id,
                                    connected: true,
                                    sender_stream: stream.try_clone().unwrap(),
                                    receiver: msg_rx,
                                })
                                .unwrap();
                            stream.set_nonblocking(false).unwrap();
                            thread::spawn(move || {
                                handle_client(stream, msg_tx);
                            });
                            debug!("New client connected: {}", client_id);
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            if shutdown_cloned.load(Ordering::Relaxed) {
                                debug!("Stopping server listener");
                                break;
                            }
                            sleep(time::Duration::from_millis(10));
                        }
                        Err(e) => warn!("Communication error: {}", e),
                    }
                }
            })
            .unwrap();
        CommsServer {
            channel_rx,
            shutdown,
        }
    }

    // Check if there is a new client connection
    pub fn try_get_channel(&self) -> Option<CommChannel> {
        let channel = self.channel_rx.try_recv();
        if channel.is_ok() {
            Some(channel.unwrap())
        } else {
            None
        }
    }

    pub fn shutdown(&mut self) {
        self.shutdown.swap(true, Ordering::Relaxed);
    }
}

fn handle_client(mut stream: TcpStream, msg_tx: Sender<ClientMessage>) {
    loop {
        match ClientMessage::deserialize_from_reader(&mut stream) {
            Ok(message) => {
                match msg_tx.send(message) {
                    Err(e) => {
                        debug!("Message receiver shutting down: {}", e);
                        break;
                    }
                    _ => {}
                };
            }
            Err(e) => {
                debug!("Message receiver shutting down: {}", e);
                break;
            }
        }
    }
}

// Communication channel with a single client
pub struct CommChannel {
    pub client_id: u32,
    pub connected: bool,
    sender_stream: TcpStream,
    receiver: Receiver<ClientMessage>,
}

impl CommChannel {
    // Receive requests from from the client, if available
    pub fn try_receive(&mut self) -> Option<ClientMessage> {
        let message = self.receiver.try_recv();
        if message.is_ok() {
            Some(message.unwrap())
        } else {
            None
        }
    }

    // Send a message to the client
    pub fn send(&mut self, message: ServerMessage) {
        if let Err(_) = message.serialize_into_writer(&mut self.sender_stream) {
            info!("Client {} disconnected", self.client_id);
            self.connected = false;
        }
    }

    // Shutdown the connection to the client
    pub fn disconnect(&mut self) {
        info!("Disconnecting client: {}", self.client_id);
        self.connected = false;
        if let Err(e) = self.sender_stream.shutdown(std::net::Shutdown::Both) {
            warn!("Error disconnecting: {:?}", e);
        }
    }
}
