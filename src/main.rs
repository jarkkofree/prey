use bevy::prelude::*;

mod player;
mod world;
mod plant;
mod deer;

use player::PlayerPlugin;
use world::WorldPlugin;
use plant::PlantPlugin;
use deer::DeerPlugin;

fn main() {
    App::new()
        
        .add_plugins((
            DefaultPlugins,
            PlayerPlugin,
            WorldPlugin,
            PlantPlugin,
            DeerPlugin,
        ))
        .run();
}

