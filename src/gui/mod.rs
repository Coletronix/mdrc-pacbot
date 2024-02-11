//! Top-level GUI elements and functionality.

use std::cell::RefMut;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use eframe::egui;
use eframe::egui::{Align, Color32, Frame, Pos2, RichText, Ui, WidgetText};
use egui_dock::{DockArea, DockState, Style};
use egui_phosphor::regular;
use pacbot_rs::game_engine::GameEngine;

fn font_setup(mut contexts: EguiContexts) {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    contexts.ctx_mut().set_fonts(fonts);
}

fn ui_system(
    mut contexts: EguiContexts,
    mut world: RefMut<World>,
    mut world_to_screen: Local<Option<Transform>>,
) {
    let mut app: Mut<App> = world.resource_mut();
    egui::Window::new("Pacbot simulation")
        .show(contexts.ctx_mut(), |f| app.update(contexts.ctx_mut(), f));
}

#[derive(Copy, Clone)]
pub enum Tab {
    Grid,
    Stopwatch,
    Unknown,
}

struct TabViewer<'a> {
    pointer_pos: Option<Pos2>,
    background_color: Color32,

    bevy_world: Option<&'a mut World>,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        match tab {
            Tab::Grid => WidgetText::from("Main Grid"),
            Tab::Stopwatch => WidgetText::from("Stopwatch"),
            _ => panic!("Widget did not declare a tab!"),
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Grid => self.grid_ui(ui),
            Tab::Stopwatch => {
                // ui.label("Particle Filter");
                // draw_stopwatch(&self.pf_stopwatch.read().unwrap(), ui, "pf_sw".to_string());
                // ui.separator();
                // ui.label("Physics");
                // draw_stopwatch(
                //     &self.physics_stopwatch.read().unwrap(),
                //     ui,
                //     "ph_sw".to_string(),
                // );
                // draw_stopwatch(&self.gui_stopwatch.read().unwrap(), ui);
            }
            _ => panic!("Widget did not declare a tab!"),
        }
    }
}

impl<'a> Default for TabViewer<'a> {
    fn default() -> Self {
        Self {
            pointer_pos: None,
            background_color: Color32::BLACK,

            bevy_world: None,
        }
    }
}

impl<'a> TabViewer<'a> {
    fn grid_ui(&mut self, ui: &mut Ui) {
        // let rect = ui.max_rect();
        // let (src_p1, src_p2) = self.selected_grid.get_soft_boundaries();
        //
        // let world_to_screen = Transform::new_letterboxed(
        //     src_p1,
        //     src_p2,
        //     Pos2::new(rect.top(), rect.left()),
        //     Pos2::new(rect.bottom(), rect.right()),
        // );
        // self.world_to_screen = Some(world_to_screen);
        // let painter = ui.painter_at(rect);
        //
        // self.draw_grid(&world_to_screen, &painter);
        //
        // if self.selected_grid == StandardGrid::Pacman {
        //     self.draw_pacman_state(&world_to_screen, &painter);
        // }

        // self.draw_simulation(&world_to_screen, &painter);
    }
}

#[derive(Clone, Debug)]
pub enum PacbotWidgetStatus {
    Ok,
    Warn(String),
    Error(String),
    NotApplicable,
}

pub trait PacbotWidget {
    fn update(&mut self) {}
    fn display_name(&self) -> &'static str;
    fn button_text(&self) -> RichText;
    fn tab(&self) -> Tab {
        Tab::Unknown
    }
    fn overall_status(&self) -> &PacbotWidgetStatus {
        &PacbotWidgetStatus::NotApplicable
    }

    fn messages(&self) -> &[(String, PacbotWidgetStatus)] {
        &[]
    }
}

/// Indicates the current meta-state of the app
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Using a game server with physics engine and recording the results to file
    Recording,
    /// Playing information back from a file; no game server but physics should still run
    Playback,
}

#[derive(Resource)]
struct App {
    tree: DockState<Tab>,
}

fn pretty_print_time_now() -> String {
    let date = chrono::Local::now();
    date.format("%Y_%m_%d__%H_%M_%S").to_string()
}

impl Default for App {
    fn default() -> Self {
        Self {
            tree: DockState::new(vec![Tab::Grid]),
        }
    }
}
impl App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {}
}

#[derive(Copy, Clone, Debug)]
pub struct GridWidget {}

impl PacbotWidget for GridWidget {
    fn display_name(&self) -> &'static str {
        "Grid"
    }

    fn button_text(&self) -> RichText {
        RichText::new(format!("{}", regular::GRID_FOUR,))
    }

    fn tab(&self) -> Tab {
        Tab::Grid
    }
}

#[derive(Clone, Debug)]
pub struct AiWidget {
    pub ai_enabled: Arc<RwLock<bool>>,
}

impl PacbotWidget for AiWidget {
    fn display_name(&self) -> &'static str {
        "AI"
    }

    fn button_text(&self) -> RichText {
        RichText::new(format!("{}", regular::BRAIN,))
    }

    fn overall_status(&self) -> &PacbotWidgetStatus {
        if *self.ai_enabled.read().unwrap() {
            &PacbotWidgetStatus::Ok
        } else {
            &PacbotWidgetStatus::NotApplicable
        }
    }
}

#[derive(Clone, Debug)]
pub struct PacbotSensorsWidget {
    pub sensors: Arc<RwLock<(bool, [u8; 8], [i64; 3], Instant)>>,

    pub overall_status: PacbotWidgetStatus,
    pub messages: Vec<(String, PacbotWidgetStatus)>,
}

impl PacbotSensorsWidget {
    pub fn new(sensors: Arc<RwLock<(bool, [u8; 8], [i64; 3], Instant)>>) -> Self {
        Self {
            sensors,

            overall_status: PacbotWidgetStatus::Ok,
            messages: vec![],
        }
    }
}

impl PacbotWidget for PacbotSensorsWidget {
    fn update(&mut self) {
        let sensors = self.sensors.read().unwrap();

        self.messages = vec![];
        self.overall_status = PacbotWidgetStatus::Ok;

        if sensors.0 {
            self.messages
                .push(("Sensors enabled".to_string(), PacbotWidgetStatus::Ok));
        } else {
            self.messages.push((
                "Sensors disabled".to_string(),
                PacbotWidgetStatus::Warn("".to_string()),
            ));
        }

        if sensors.3.elapsed() > Duration::from_secs(1) {
            self.messages.push((
                format!("Last data age: {:?}", sensors.3.elapsed()),
                PacbotWidgetStatus::Error("".to_string()),
            ));
            self.overall_status =
                PacbotWidgetStatus::Error(format!("Last data age: {:?}", sensors.3.elapsed()));
        } else {
            for i in 0..8 {
                if sensors.1[i] == 0 {
                    self.messages.push((
                        format!("Sensor {i} unresponsive"),
                        PacbotWidgetStatus::Error("".to_string()),
                    ));
                    self.overall_status =
                        PacbotWidgetStatus::Error(format!("Sensor {i} unresponsive"));
                }
                self.messages.push((
                    format!("{i} => {}", sensors.1[i]),
                    match sensors.1[i] {
                        0 => PacbotWidgetStatus::Error("".to_string()),
                        255 => PacbotWidgetStatus::Warn("".to_string()),
                        _ => PacbotWidgetStatus::Ok,
                    },
                ))
            }
            for i in 0..3 {
                self.messages.push((
                    format!("Encoder {i}: {}", sensors.2[i]),
                    PacbotWidgetStatus::Ok,
                ));
            }
        }
    }

    fn display_name(&self) -> &'static str {
        "Sensors"
    }

    fn button_text(&self) -> RichText {
        RichText::new(format!("{}", regular::RULER,))
    }

    fn overall_status(&self) -> &PacbotWidgetStatus {
        &self.overall_status
    }

    fn messages(&self) -> &[(String, PacbotWidgetStatus)] {
        &self.messages
    }
}
