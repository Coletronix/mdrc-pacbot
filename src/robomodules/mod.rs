use anyhow::Error;
use byteorder::{BigEndian, ReadBytesExt};
use protobuf::Message;
use std::io::{Cursor, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

pub mod message;
pub mod protos;

use message::MAGIC_HEADER;

use crate::robomodules::message::{write_message, Msg};
use message::MsgType;

const SIZE_HEADER: usize = 12; // size of two u16s + one u64

enum ClientCommand {
    Close,
    Subscribe(MsgType),
    Unsubscribe(MsgType),
}

pub struct Client {
    handler: JoinHandle<()>,
    command_sender: Sender<ClientCommand>,
    pub msg_receiver: Receiver<Msg>,
}

struct ThreadedClient {
    stream: TcpStream,

    receiver: Receiver<ClientCommand>,
    sender: Sender<Msg>,

    subscriptions: [u16; 3],

    buffer: Vec<u8>,
    msg_type: u16,
    msg_length: usize,
}

impl Client {
    pub fn new(addr: &str, port: u16) -> Self {
        let addr = addr.to_owned();
        let (command_sender, receiver) = channel();
        let (sender, msg_receiver) = channel();
        let handler = thread::spawn(move || {
            let threaded_client = ThreadedClient::new(addr, port, sender, receiver).unwrap();

            threaded_client.run();
        });

        Self {
            handler,
            command_sender,
            msg_receiver,
        }
    }

    pub fn subscribe(&mut self, msg_type: MsgType) -> Result<(), Error> {
        self.command_sender
            .send(ClientCommand::Subscribe(msg_type))?;

        Ok(())
    }

    pub fn unsubscribe(&mut self, msg_type: MsgType) -> Result<(), Error> {
        self.command_sender
            .send(ClientCommand::Unsubscribe(msg_type))?;

        Ok(())
    }

    pub fn close(self) -> Result<(), Error> {
        self.command_sender.send(ClientCommand::Close)?;
        match self.handler.join() {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::msg("Failed to join thread")),
        }
    }
}

impl ThreadedClient {
    pub fn new(
        addr: String,
        port: u16,
        sender: Sender<Msg>,
        receiver: Receiver<ClientCommand>,
    ) -> Result<Self, Error> {
        let stream = TcpStream::connect(format!("{}:{}", addr, port))?;
        stream.set_nonblocking(true)?;

        Ok(Self {
            stream,

            receiver,
            sender,

            subscriptions: [0; 3],

            buffer: Vec::new(),
            msg_type: 0,
            msg_length: 0,
        })
    }

    pub fn run(mut self) {
        loop {
            // first process all waiting commands
            while let Ok(command) = self.receiver.try_recv() {
                match command {
                    ClientCommand::Close => {
                        return;
                    }
                    ClientCommand::Subscribe(msg_type) => {
                        if self.subscriptions[msg_type as usize] == 0 {
                            self.subscriptions[msg_type as usize] = 1;

                            let mut sub = protos::subscribe::Subscribe::new();
                            sub.set_dir(protos::subscribe::Subscribe_Direction::SUBSCRIBE);
                            sub.set_msg_types(vec![msg_type as u16 as i32]);

                            self.stream
                                .write(write_message(Msg::Subscribe(sub)).as_slice())
                                .unwrap();
                        }
                    }
                    ClientCommand::Unsubscribe(msg_type) => {
                        if self.subscriptions[msg_type as usize] == 1 {
                            self.subscriptions[msg_type as usize] = 0;

                            let mut sub = protos::subscribe::Subscribe::new();
                            sub.set_dir(protos::subscribe::Subscribe_Direction::UNSUBSCRIBE);
                            sub.set_msg_types(vec![msg_type as u16 as i32]);

                            self.stream
                                .write(write_message(Msg::Subscribe(sub)).as_slice())
                                .unwrap();
                        }
                    }
                }
            }

            // check stream for bytes - send any to data_received
            let mut buf = [0u8; 4];
            match self.stream.read(&mut buf) {
                Ok(_) => {
                    self.data_received(&buf);
                }
                Err(_) => {}
            }
        }
    }

    fn data_received(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);

        while !self.buffer.is_empty() {
            if self.msg_length == 0 && self.buffer.len() >= SIZE_HEADER {
                let mut cursor = Cursor::new(&self.buffer);
                let magic = cursor.read_u16::<BigEndian>().unwrap();
                self.msg_type = cursor.read_u16::<BigEndian>().unwrap();
                self.msg_length = cursor.read_u64::<BigEndian>().unwrap() as usize; // Convert u64 to usize

                // Remove the processed header from the buffer
                self.buffer.drain(0..SIZE_HEADER);

                if magic != MAGIC_HEADER {
                    // reset
                    self.buffer.clear();
                    self.msg_length = 0;
                    self.msg_type = 0;
                }
            } else if self.msg_length != 0 && self.buffer.len() >= self.msg_length {
                let msg = self.buffer[..self.msg_length].to_vec();
                if self.subscriptions[self.msg_type as usize] == 1 {
                    self.msg_received(&msg, self.msg_type);
                }

                // Remove the processed message from the buffer
                self.buffer.drain(0..self.msg_length);
                self.msg_length = 0;
                self.msg_type = 0;
            } else {
                return;
            }
        }
    }

    fn msg_received(&mut self, msg: &[u8], msg_type: u16) {
        match msg_type {
            0 => {
                let light_state = protos::light_state::LightState::parse_from_bytes(msg).unwrap();
                self.sender.send(Msg::LightState(light_state)).unwrap();
            }
            1 => {
                let pacman_location =
                    protos::pacman_state::PacmanState_AgentState::parse_from_bytes(msg).unwrap();
                self.sender
                    .send(Msg::PacmanLocation(pacman_location))
                    .unwrap();
            }
            2 => {
                let full_state = protos::pacman_state::PacmanState::parse_from_bytes(msg).unwrap();
                self.sender.send(Msg::FullState(full_state)).unwrap();
            }
            _ => {}
        }
    }
}
