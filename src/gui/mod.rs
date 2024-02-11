//! Top-level GUI elements and functionality.

use std::cell::RefMut;

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use eframe::egui;
use eframe::egui::{RichText, Ui, WidgetText};
use egui_dock::DockState;

fn ui_system(mut contexts: EguiContexts, mut world: RefMut<World>) {
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

struct TabViewer {}

impl egui_dock::TabViewer for TabViewer {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        match tab {
            Tab::Grid => WidgetText::from("Main Grid"),
            Tab::Stopwatch => WidgetText::from("Stopwatch"),
            _ => panic!("Widget did not declare a tab!"),
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {}
}

impl Default for TabViewer {
    fn default() -> Self {
        Self {}
    }
}

impl TabViewer {
    fn grid_ui(&mut self, ui: &mut Ui) {}
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

#[derive(Resource)]
struct App {
    tree: DockState<Tab>,
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
