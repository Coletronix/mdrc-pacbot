use bevy::prelude::{Mut, Resource, World};
use bevy_egui::egui::{Ui, WidgetText};
use bevy_egui::{egui, EguiContexts};
use std::cell::RefMut;

fn ui_system(mut contexts: EguiContexts, mut world: RefMut<World>) {
    let mut app: Mut<App> = world.resource_mut();
    egui::Window::new("Pacbot simulation")
        .show(contexts.ctx_mut(), |f| app.update(contexts.ctx_mut(), f));
}

#[derive(Default)]
struct TabViewer {}

impl egui_dock::TabViewer for TabViewer {
    type Tab = ();

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        WidgetText::from("Stopwatch")
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {}
}

pub trait PacbotWidget {
    fn update(&mut self) {}
}

#[derive(Resource)]
struct App {}

impl Default for App {
    fn default() -> Self {
        Self {}
    }
}
impl App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {}
}

fn main() {
    println!("Hello world!")
}
