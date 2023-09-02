use crate::robomodules::protos;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use protobuf::Message;

pub const MAGIC_HEADER: u16 = 17380;

#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum MsgType {
    LightState = 0,
    PacmanLocation = 1,
    FullState = 2,
    Subscribe = 15000,
}

#[derive(Debug)]
pub enum Msg {
    LightState(protos::light_state::LightState),
    PacmanLocation(protos::pacman_state::PacmanState_AgentState),
    FullState(protos::pacman_state::PacmanState),
    Subscribe(protos::subscribe::Subscribe),
}

pub fn write_message(message: Msg) -> Vec<u8> {
    let mut msg = Vec::new();

    msg.extend_from_slice(&MAGIC_HEADER.to_be_bytes());

    let mut bytes = Vec::new();
    let msg_type = match message {
        Msg::LightState(m) => {
            m.write_to_vec(&mut bytes).unwrap();
            MsgType::LightState
        }
        Msg::PacmanLocation(m) => {
            m.write_to_vec(&mut bytes).unwrap();
            MsgType::PacmanLocation
        }
        Msg::FullState(m) => {
            m.write_to_vec(&mut bytes).unwrap();
            MsgType::FullState
        }
        Msg::Subscribe(m) => {
            m.write_to_vec(&mut bytes).unwrap();
            MsgType::Subscribe
        }
    };

    msg.extend_from_slice(&(msg_type as u16).to_be_bytes());
    msg.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    msg.extend_from_slice(&bytes);

    msg
}
