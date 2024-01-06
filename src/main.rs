use bevy::prelude::*;

mod player;
mod world;
mod plant;

use player::PlayerPlugin;
use world::WorldPlugin;
use plant::PlantPlugin;

fn main() {
    App::new()
        
        .add_plugins((
            DefaultPlugins,
            PlayerPlugin,
            WorldPlugin,
            PlantPlugin,
        ))
        .run();
}

