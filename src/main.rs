use bevy::prelude::*;
use pacbot_rs::game_engine::GameEngine;

pub mod gui;

/// The state of Pacman, the game
#[derive(Default, Resource)]
pub struct PacmanGameState(GameEngine);

/// Options that the user can set via the GUI, shared between most processes
#[derive(Resource)]
pub struct UserSettings {
    pub enable_ai: bool,
    pub enable_pico: bool,
    pub pico_address: String,

    pub replay_save_location: bool,
    pub replay_save_sensors: bool,
    pub replay_save_targets: bool,
}

fn main() {
    println!("Hello world!")
}
