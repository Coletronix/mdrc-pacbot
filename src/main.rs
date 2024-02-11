use bevy::prelude::{Mut, Resource, World};
use eframe::egui::Ui;

fn ui_system(mut world: &World, f: &mut Ui) {
    let mut app: Mut<App> = world.resource_mut();
    app.update(f);
}

pub trait UselessTrait {
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
