use bevy::prelude::{Mut, Resource, World};
use bevy_egui::{egui, EguiContexts};

fn ui_system(mut contexts: EguiContexts, mut world: &World) {
    let mut app: Mut<App> = world.resource_mut();
    egui::Window::new("Pacbot simulation").show(contexts.ctx_mut(), |f| app.update(f));
}

pub trait PacbotWidget {
    fn update(&mut self) {}
}

#[derive(Resource, Default)]
struct App {}

impl App {
    fn update(&mut self, _: &mut eframe::Frame) {}
}

fn main() {
    println!("Hello world!")
}
