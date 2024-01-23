//! Defines the Pacman agent's high level AI.

use crate::grid::standard_grids::StandardGrid;
use crate::grid::ComputedGrid;
use crate::grid::PLocation;
use candle_core::{Device, Module, Tensor};
use candle_nn as nn;
use ndarray::{s, Array};
use pacbot_rs::game_modes::GameMode;
use pacbot_rs::game_state::GameState;
use pacbot_rs::variables;
use pacbot_rs::variables::GHOST_FRIGHT_STEPS;

/// Represents an action the AI can choose to perform.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HLAction {
    /// The agent should stay in place.
    Stay,
    /// The agent should move left.
    Left,
    /// The agent should move right.
    Right,
    /// The agent should move up.
    Up,
    /// The agent should move down.
    Down,
}

const OBS_SHAPE: (usize, usize, usize) = (15, 28, 31);

/// Handles executing high level AI.
pub struct HighLevelContext {
    net: QNetV2,
    last_pos: (usize, usize),
    last_ghost_pos: Vec<(usize, usize)>,
}

impl HighLevelContext {
    /// Creates a new instance of the high level AI.
    pub fn new(weights_path: &str) -> Self {
        let mut vm = nn::VarMap::new();
        let vb =
            nn::VarBuilder::from_varmap(&vm, candle_core::DType::F32, &candle_core::Device::Cpu);
        let net = QNetV2::new(
            candle_core::Shape::from_dims(&[OBS_SHAPE.0, OBS_SHAPE.1, OBS_SHAPE.2]),
            5,
            vb,
        )
        .unwrap();
        vm.load(weights_path).unwrap();
        Self {
            net,
            last_pos: (0, 0),
            last_ghost_pos: vec![(0, 0), (0, 0), (0, 0), (0, 0)],
        }
    }

    /// Runs one step of the high level AI.
    /// Returns the action the AI has decided to take.
    // Currently, this implements a DQN approach.
    pub fn step(&mut self, game_state: &GameState, grid: &ComputedGrid) -> HLAction {
        // Convert the current game state into an agent observation.
        let mut obs_array = Array::zeros(OBS_SHAPE);
        let (mut wall, mut reward, mut pacman, mut ghost, mut last_ghost, mut state) = obs_array
            .multi_slice_mut((
                s![0, .., ..],
                s![1, .., ..],
                s![2..4, .., ..],
                s![4..8, .., ..],
                s![8..12, .., ..],
                s![12..15, .., ..],
            ));
        
        for row in 0..31 {
            for col in 0..28 {
                let obs_row = 31 - row - 1;
                wall[(col, obs_row)] = grid.grid()[row][col] as u8 as f32;
                reward[(col, obs_row)] = if game_state.pellet_at((row as i8, col as i8)) {
                    if ((row == 3) || (row == 23)) && ((col == 1) || (col == 26)) {
                        variables::SUPER_PELLET_POINTS
                    } else {
                        variables::PELLET_POINTS
                    }
                } else if game_state.fruit_exists()
                    && col == game_state.fruit_loc.col as usize
                    && row == game_state.fruit_loc.row as usize
                {
                    variables::FRUIT_POINTS
                } else {
                    0
                } as f32
                    / variables::COMBO_MULTIPLIER as f32;
            }
        }

        let pac_pos = game_state.pacman_loc;

        // I think (32, 32) is the shadow realm
        if pac_pos.col != 32 && self.last_pos.0 != 32 {
            pacman[(0, self.last_pos.0, self.last_pos.1)] = 1.0;
            pacman[(1, pac_pos.col as usize, pac_pos.row as usize)] = 1.0;

            for (i, g) in game_state.ghosts.iter().enumerate() {
                let g = g.read().unwrap();
                let pos = g.loc;
                if pos.col != 32 {
                    ghost[(i, pos.col as usize, pos.row as usize)] = 1.0;
                    if g.is_frightened() {
                        state[(2, pos.col as usize, pos.row as usize)] =
                            g.fright_steps as f32 / GHOST_FRIGHT_STEPS as f32;
                    } else {
                        let state_index = if game_state.mode == GameMode::CHASE {
                            1
                        } else {
                            0
                        };
                        state[(state_index, pos.col as usize, pos.row as usize)] = 1.0;
                    }
                }
            }
        }

        for (i, pos) in self.last_ghost_pos.iter().enumerate() {
            if pos.0 != 32 {
                last_ghost[(i, pos.0, pos.1)] = 1.0;
            }
        }

        // Save last positions.
        self.last_pos = (pac_pos.col as usize, pac_pos.row as usize);
        self.last_ghost_pos = game_state
            .ghosts
            .iter()
            .map(|g| g.read().unwrap())
            .map(|g| (g.loc.col as usize, g.loc.row as usize))
            .collect();

        // Create action mask.
        let mut action_mask = [true, false, false, false, false];
        if let Some(valid_actions) = grid.valid_actions(PLocation::new(pac_pos.row, pac_pos.col)) {
            action_mask = [
                true, // !valid_actions[0],
                !valid_actions[4],
                !valid_actions[3],
                !valid_actions[2],
                !valid_actions[1],
            ];
        }
        let action_mask =
            Tensor::from_slice(&action_mask.map(|b| b as u8 as f32), 5, &Device::Cpu).unwrap(); // 1 if masked, 0 if not

        // Run observation through model and generate action.
        let obs_flat = obs_array.as_slice().unwrap();
        let obs_tensor = Tensor::from_slice(obs_flat, OBS_SHAPE, &Device::Cpu)
            .unwrap()
            .unsqueeze(0)
            .unwrap()
            .to_dtype(candle_core::DType::F32)
            .unwrap();
        let q_vals = self.net.forward(&obs_tensor).unwrap().squeeze(0).unwrap();
        let q_vals = ((q_vals * (1. - &action_mask).unwrap()).unwrap()
            + (&action_mask * -999.).unwrap())
        .unwrap();
        let actions = [
            HLAction::Stay,
            HLAction::Down,
            HLAction::Up,
            HLAction::Left,
            HLAction::Right,
        ];
        actions[q_vals
            .argmax(candle_core::D::Minus1)
            .unwrap()
            .to_scalar::<u32>()
            .unwrap() as usize]
    }
}

/// Returns a convolutional block.
fn conv_block_pool(
    in_channels: usize,
    out_channels: usize,
    vb: nn::VarBuilder,
) -> candle_core::Result<nn::Sequential> {
    Ok(nn::seq()
        .add(nn::conv2d(
            in_channels,
            out_channels,
            3,
            nn::Conv2dConfig {
                padding: 1,
                ..Default::default()
            },
            vb,
        )?)
        .add(nn::func(|x| x.max_pool2d(2)))
        .add(nn::Activation::Silu))
}

/// The Q network.
struct QNetV2 {
    backbone: nn::Sequential,
    value_head: nn::Sequential,
    advantage_head: nn::Sequential,
}

impl QNetV2 {
    pub fn new(
        obs_shape: candle_core::Shape,
        action_count: usize,
        vb: nn::VarBuilder,
    ) -> candle_core::Result<Self> {
        let (obs_channels, _, _) = obs_shape.dims3().unwrap();
        let b_vb = vb.pp("backbone");
        let backbone = nn::seq()
            .add(nn::conv2d(
                obs_channels,
                16,
                5,
                nn::Conv2dConfig {
                    padding: 2,
                    ..Default::default()
                },
                b_vb.pp("0"),
            )?)
            .add(nn::Activation::Silu)
            .add(conv_block_pool(16, 32, b_vb.pp("2"))?)
            .add(conv_block_pool(32, 64, b_vb.pp("5"))?)
            .add(conv_block_pool(64, 128, b_vb.pp("8"))?)
            .add(nn::conv2d(
                128,
                128,
                3,
                nn::Conv2dConfig {
                    padding: 1,
                    groups: 128 / 16,
                    ..Default::default()
                },
                b_vb.pp("11"),
            )?)
            .add_fn(|xs| xs.max(candle_core::D::Minus1)?.max(candle_core::D::Minus1))
            .add(nn::func(|x| x.flatten(1, candle_core::D::Minus1)))
            .add(nn::Activation::Silu)
            .add(nn::linear(128, 256, b_vb.pp("15"))?)
            .add(nn::Activation::Silu);
        let value_head = nn::seq().add(nn::linear(256, 1, vb.pp("value_head").pp("0"))?);
        let advantage_head = nn::seq().add(nn::linear(
            256,
            action_count,
            vb.pp("advantage_head").pp("0"),
        )?);

        Ok(Self {
            backbone,
            value_head,
            advantage_head,
        })
    }
}

impl Module for QNetV2 {
    fn forward(&self, input_batch: &Tensor) -> candle_core::Result<Tensor> {
        let backbone_features = self.backbone.forward(input_batch)?;
        let value = self.value_head.forward(&backbone_features)?;
        let advantages = self.advantage_head.forward(&backbone_features)?;
        value.broadcast_sub(&advantages.mean_keepdim(1)?.broadcast_add(&advantages)?)
    }
}
