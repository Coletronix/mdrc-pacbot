//! Network communications with the Pico and the game server.

use crate::pathing::TargetVelocity;
use crate::physics::LightPhysicsInfo;
use crate::UserSettings;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::FRAC_PI_3;
use std::{io, net::UdpSocket};
use tokio::net::TcpStream;

/// Stores data from Pacbot
#[derive(Resource, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PacbotSensors {
    pub distance_sensors: [u8; 8],
    pub encoders: [i64; 3],
}

impl Default for PacbotSensors {
    fn default() -> Self {
        Self {
            distance_sensors: [0; 8],
            encoders: [0; 3],
        }
    }
}

/// Stores connections for the NetworkPlugin
#[derive(Default, Resource)]
pub struct NetworkPluginData {
    pico: Option<PicoConnection>,
    game_server: Option<TcpStream>,
}

/// Network communications with the Pico and the game server.
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                reconnect_pico,
                send_motor_commands.after(reconnect_pico),
                recv_pico.after(reconnect_pico),
            ),
        );
    }
}

fn send_motor_commands(
    mut network_data: ResMut<NetworkPluginData>,
    target_velocity: Res<TargetVelocity>,
    phys_info: Res<LightPhysicsInfo>,
) {
    if let Some(pico) = &mut network_data.pico {
        if let Some(loc) = phys_info.pf_pos {
            let x = target_velocity.0.x;
            let y = target_velocity.0.y;

            let current_angle = loc.rotation.angle();

            // use x and y to find the desired angle
            let angle = y.atan2(x);
            let angle = angle - current_angle;

            let mut scale = (x.powi(2) + y.powi(2)).sqrt();
            if scale != 0.0 {
                scale = 7.0;
            }

            let motor_angles = [
                angle.cos(),
                (angle - (2.0 * FRAC_PI_3)).cos(),
                (angle + (2.0 * FRAC_PI_3)).cos(),
            ];

            let rotate_adjust = if target_velocity.1 > 0.0 {
                3.0
            } else if target_velocity.1 < 0.0 {
                -3.0
            } else {
                0.0
            };

            let motors_i16 = [
                // constant is like max speed - can go up to 255.0
                (motor_angles[0] * (255.0 / 11.6) * scale + rotate_adjust * (255.0 / 11.6)) as i16,
                (motor_angles[1] * (255.0 / 11.6) * scale + rotate_adjust * (255.0 / 11.6)) as i16,
                (motor_angles[2] * (255.0 / 11.6) * scale + rotate_adjust * (255.0 / 11.6)) as i16,
            ];

            let mut motors = [(0, true); 3];
            for i in 0..3 {
                motors[i].0 = motors_i16[i].abs() as u8;
                motors[i].1 = motors_i16[i] >= 0;
            }

            if let Err(e) = pico.send_motors_message(motors) {
                eprintln!("{:?}", e);
                network_data.pico = None;
            }
        } else {
            let motors = [(0, true); 3];

            if let Err(e) = pico.send_motors_message(motors) {
                eprintln!("{:?}", e);
                network_data.pico = None;
            }
        }
    }
}

fn reconnect_pico(mut network_data: ResMut<NetworkPluginData>, settings: Res<UserSettings>) {
    if network_data.pico.is_none() {
        if let Some(pico_address) = &settings.pico_address {
            let try_conn = PicoConnection::new(20001, &pico_address);
            if let Err(ref e) = try_conn {
                eprintln!("{:?}", e);
            }
            network_data.pico = try_conn.ok();
            if let Some(pico) = &mut network_data.pico {
                if let Err(e) = pico.socket.set_nonblocking(true) {
                    eprintln!("{:?}", e);
                    network_data.pico = None;
                }
            }
        }
    }
}

fn recv_pico(mut network_data: ResMut<NetworkPluginData>, mut sensors: ResMut<PacbotSensors>) {
    if let Some(pico) = &mut network_data.pico {
        let mut bytes = [0; 30];
        while let Ok(size) = pico.socket.recv(&mut bytes) {
            if size == 20 {
                for i in 0..8 {
                    sensors.distance_sensors[i] = bytes[i];
                }
                for i in 0..3 {
                    sensors.encoders[i] = i32::from_le_bytes([
                        bytes[i * 4 + 8],
                        bytes[i * 4 + 9],
                        bytes[i * 4 + 10],
                        bytes[i * 4 + 11],
                    ]) as i64;
                }
            } else {
                eprintln!("Invalid message size from Pico: {size}");
            }
        }
    }
}

/// Types of messages sent to the Pico.
#[repr(u8)]
enum MessageType {
    Motors = 1,
    Sleep = 2,
    AccelOffset = 3,
    DistanceOffset = 4,
}

struct PicoConnection {
    socket: UdpSocket,
    next_ack: u16,
}

#[allow(dead_code)]
impl PicoConnection {
    fn new(local_port: u16, remote_address: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(("0.0.0.0", local_port))?;
        socket.connect(remote_address)?;
        Ok(Self {
            socket,
            next_ack: 1,
        })
    }

    fn send_message(&mut self, message: &[u8]) -> io::Result<()> {
        self.socket.send(message)?;
        Ok(())
    }

    fn send_motors_message(&mut self, motors: [(u8, bool); 3]) -> io::Result<()> {
        let mut message = [0; 7];
        message[0] = MessageType::Motors as u8;
        message[1] = motors[0].0;
        message[2] = motors[1].0;
        message[3] = motors[2].0;
        message[4] = if motors[0].1 { 2 } else { 0 };
        message[5] = if motors[1].1 { 2 } else { 0 };
        message[6] = if motors[2].1 { 2 } else { 0 };
        self.socket.set_nonblocking(false).unwrap();
        let r = self.send_message(&message);
        self.socket.set_nonblocking(true).unwrap();
        r
    }
}