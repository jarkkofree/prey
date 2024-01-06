use bevy::prelude::*;
use rand::prelude::*;

pub struct PlantPlugin;

impl Plugin for PlantPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(Update, spawn_plants);
    }
}

#[derive(Resource)]
struct PlantConfig {

    plant_height: f32,
    plant_radius: f32,
    plant_color: Color,

    spawn_rate: f32,
    spawn_chance: f32,
    spawn_range: f32,
    spawn_max: usize,
    spawn_area: f32,
}

impl Default for PlantConfig {
    fn default() -> Self {
        PlantConfig {
            plant_height: 1.0,
            plant_radius: 0.15,
            plant_color: Color::rgb(0.3, 0.7, 0.1),
            spawn_rate: 0.25,
            spawn_chance: 0.7, // 1/count^spawn_chance
            spawn_range: 10.0,
            spawn_max: 5000,
            spawn_area: 25.0,
        }
    }
}

#[derive(Resource)]
struct PlantAssets {
    mesh: Mesh,
    material: StandardMaterial,
}

impl PlantAssets {
    fn new(config: &PlantConfig) -> Self {
        PlantAssets {
            mesh: Mesh::from(
                shape::Box::new(
                    config.plant_radius,
                    config.plant_height,
                    config.plant_radius
                )
            ),
            material: StandardMaterial {
                base_color: config.plant_color,
                ..Default::default()
            },
        }
    }
}

#[derive(Resource)]
struct PlantHandles {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

struct Plant {
    x: f32,
    y: f32,
    height: f32,
}

impl Plant {
    fn new(x: f32, y: f32, config: &PlantConfig) -> Self {

        Plant {
            x,
            y,
            height: config.plant_height,
        }
    }

    fn transform(&self) -> Transform {
        const HALF: f32 = 0.5;
        Transform::from_xyz(self.x, self.height*HALF, self.y)
    }
}

#[derive(Component)]
struct PlantTag;

#[derive(Resource)]
struct PlantSpawn {
    timer: Timer,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let config = PlantConfig::default();

    let plant_assets = PlantAssets::new(&config);
    let plant_handles = PlantHandles {
        mesh: meshes.add(plant_assets.mesh.clone()),
        material: materials.add(plant_assets.material.clone()),
    };

    commands.insert_resource(plant_assets);
    commands.insert_resource(plant_handles);

    commands.insert_resource(PlantSpawn {
        timer: Timer::from_seconds(config.spawn_rate, TimerMode::Repeating),
    });

    commands.insert_resource(config);

}

fn spawn_plants(
    mut commands: Commands,
    plants: Query<(&Transform, With<PlantTag>)>,
    mut plant_spawn: ResMut<PlantSpawn>,
    time: Res<Time>,
    config: Res<PlantConfig>,
    plant_handles: Res<PlantHandles>,
) {

    plant_spawn.timer.tick(time.delta());

    if plant_spawn.timer.finished() {

        let mut new_plants: Vec<Plant> = Vec::new();

        let plant_count = plants.iter().count();

        if plant_count == 0 {

            let plant = Plant::new(
                0.0,
                0.0,
                &config
            );
            new_plants.push(plant);

        } else if plant_count < config.spawn_max {
            let mut rng = rand::thread_rng();

            for (t, _) in plants.iter() {
    
                let threshold = 1.0 / (plant_count as f32).powf(config.spawn_chance);
                let fail_chance = rng.gen_range(0.0..1.0);
                if fail_chance > threshold {
                    continue;
                }
    
                let x = rng.gen_range(-config.spawn_range..config.spawn_range) + t.translation.x;
                let y = rng.gen_range(-config.spawn_range..config.spawn_range) + t.translation.z;
    
                if (x.abs()).max(y.abs()) > config.spawn_area {
                    info!("out of bounds: x: {}, y: {}", x, y);
                    continue;
                }
    
                let plant = Plant::new(
                    x,
                    y,
                    &config
                );
                new_plants.push(plant);
            }

        } else {
            info!("max: {}", plant_count);
        }

        for p in new_plants.iter() {
            commands.spawn((
                PbrBundle {
                    mesh: plant_handles.mesh.clone(),
                    material: plant_handles.material.clone(),
                    transform: p.transform(),
                    ..Default::default()
                },
                PlantTag,
            ));
        }

    }

}