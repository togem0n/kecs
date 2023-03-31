// pub mod entity;
// pub mod entity_manager;
// pub mod entity_builder;
// pub mod component;
// pub mod component_manager;
// pub mod ecs;
// pub mod system;
// pub mod system_manager;
// pub mod printing;

pub mod test_dir;
use crate::test_dir::world::{*, self};

fn main() {
    println!("Hello, world!");
    let mut world = World::new();
    struct Name {};
    struct Move {};
    world.spawn((21, true));
    world.spawn((2, false));
    world.spawn((2, true));
    world.spawn((5, true));
    world.spawn((21, Name{}, Move{}));
    println!("{}", world.archetypes.len());
    println!("{}", world.entities.len());
}
