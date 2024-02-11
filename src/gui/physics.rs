use crate::gui::colors::{
    PACMAN_COLOR, PACMAN_DISTANCE_SENSOR_RAY_COLOR, PACMAN_FACING_INDICATOR_COLOR,
    PACMAN_GUESS_COLOR, PACMAN_REPLAY_COLOR,
};
use crate::gui::transforms::Transform;
use crate::gui::{AppMode, TabViewer};
use crate::physics::LightPhysicsInfo;
use crate::replay_manager::ReplayManager;
use crate::robot::Robot;
use crate::UserSettings;
use eframe::egui::{Painter, Pos2, Stroke};
use rapier2d::na::Point2;

impl<'a> TabViewer<'a> {
    pub(super) fn draw_simulation(
        &mut self,
        world_to_screen: &Transform,
        painter: &Painter,
        phys_info: &LightPhysicsInfo,
        replay: &ReplayManager,
        settings: &UserSettings,
    ) {
        let collider_radius = Robot::default().collider_radius;

        // pacbot real position
        if let Some(real_pos) = &phys_info.real_pos {
            painter.circle_filled(
                world_to_screen
                    .map_point(Pos2::new(real_pos.translation.x, real_pos.translation.y)),
                world_to_screen.map_dist(collider_radius),
                PACMAN_COLOR,
            );
        }

        // pacbot best estimate position
        if let Some(pf_pos) = &phys_info.pf_pos {
            painter.circle_stroke(
                world_to_screen.map_point(Pos2::new(pf_pos.translation.x, pf_pos.translation.y)),
                world_to_screen.map_dist(collider_radius),
                Stroke::new(2.0, PACMAN_GUESS_COLOR),
            );
        }

        if let Some(real_pos) = &phys_info.real_pos {
            let pacbot_front = real_pos.rotation.transform_point(&Point2::new(0.45, 0.0));

            // pacbot facing indicator
            painter.line_segment(
                [
                    world_to_screen
                        .map_point(Pos2::new(real_pos.translation.x, real_pos.translation.y)),
                    world_to_screen.map_point(Pos2::new(
                        pacbot_front.x + real_pos.translation.x,
                        pacbot_front.y + real_pos.translation.y,
                    )),
                ],
                Stroke::new(2.0, PACMAN_FACING_INDICATOR_COLOR),
            );
        }

        let replay_pacman = replay.replay.get_pacbot_location();

        // pacbot from the replay
        if matches!(settings.mode, AppMode::Playback) {
            painter.circle_filled(
                world_to_screen.map_point(Pos2::new(
                    replay_pacman.translation.x,
                    replay_pacman.translation.y,
                )),
                world_to_screen.map_dist(collider_radius),
                PACMAN_REPLAY_COLOR,
            );

            let pacbot_front = replay_pacman
                .rotation
                .transform_point(&Point2::new(0.45, 0.0));

            painter.line_segment(
                [
                    world_to_screen.map_point(Pos2::new(
                        replay_pacman.translation.x,
                        replay_pacman.translation.y,
                    )),
                    world_to_screen.map_point(Pos2::new(
                        pacbot_front.x + replay_pacman.translation.x,
                        pacbot_front.y + replay_pacman.translation.y,
                    )),
                ],
                Stroke::new(2.0, PACMAN_FACING_INDICATOR_COLOR),
            );
        }

        // pacbot best guess distance sensor rays
        for (s, f) in &phys_info.pf_pos_rays {
            painter.line_segment(
                [
                    world_to_screen.map_point(Pos2::new(s.x, s.y)),
                    world_to_screen.map_point(Pos2::new(f.x, f.y)),
                ],
                Stroke::new(1.0, PACMAN_DISTANCE_SENSOR_RAY_COLOR),
            );
        }

        // TODO
        // particle filter
        // let pf_points = &phys_render.pf_points;
        //
        // for p in pf_points {
        //     painter.circle_filled(
        //         world_to_screen.map_point(Pos2::new(p.translation.x, p.translation.y)),
        //         1.0,
        //         PACMAN_PARTICLE_FILTER_COLOR,
        //     );
        // }

        // AI target position
        // if *self.ai_enable.read().unwrap() {
        //     let target = *self.target_pos.read().unwrap();
        //     painter.line_segment(
        //         [
        //             world_to_screen.map_point(Pos2::new(
        //                 pacbot_pos.translation.x,
        //                 pacbot_pos.translation.y,
        //             )),
        //             world_to_screen.map_point(Pos2::new(target.0 as f32, target.1 as f32)),
        //         ],
        //         Stroke::new(2.0, PACMAN_AI_TARGET_LOCATION_COLOR),
        //     );
        // }
    }
}
