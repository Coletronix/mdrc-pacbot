use bevy::prelude::{Mut, Resource, World};

pub trait UselessTrait {
    fn update(&mut self) {}
}

#[derive(Resource, Default)]
struct App {}

impl App {
    fn update(&mut self, _: &mut i32) {}
}

fn main() {
    let mut world = World::new();
    let mut app: Mut<App> = world.resource_mut();
    app.update(0u32);
}
