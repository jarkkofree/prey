use bevy::prelude::*;
use rand::prelude::*;

use crate::plant::PlantTag;

pub struct DeerPlugin;

impl Plugin for DeerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(Update, (
                spawn,
                deer_ai,
            ));
    }
}

#[derive(Resource)]
struct Config {
    color: Color,
    height: f32,
    torso_length: f32,
    torso_height: f32,
    torso_width: f32,
    spawn_area: f32,
    spawn_rate: isize,
    speed: f32,
    search_radius: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            color: Color::rgb(0.4, 0.2, 0.1),
            height: 2.0,
            torso_length: 2.0,
            torso_height: 1.0,
            torso_width: 1.0,
            spawn_area: 50.0,
            spawn_rate: 10,
            speed: 1.0,
            search_radius: 5.0,
        }
    }
}

impl Config {
    fn get_pos(&self) -> Vec2 {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-self.spawn_area..self.spawn_area);
        let y = rng.gen_range(-self.spawn_area..self.spawn_area);
        Vec2::new(x, y)
    }
}

#[derive(Resource)]
struct DeerAssets {
    mesh: Mesh,
    material: StandardMaterial,
}

impl DeerAssets {
    fn new(config: &Config) -> Self {
        DeerAssets {
            mesh: Mesh::from(
                shape::Box::new(
                    config.torso_width,
                    config.torso_height,
                    config.torso_length,
                )
            ),
            material: StandardMaterial {
                base_color: config.color,
                ..Default::default()
            },
        }
    }
}

#[derive(Resource)]
struct DeerAssetHandles {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

struct Deer {
    x: f32,
    y: f32,
    height: f32,
}

impl Deer {
    fn new(config: &Config) -> Self {

        let pos = config.get_pos();

        Deer {
            x: pos.x,
            y: pos.y,
            height: config.height,
        }
    }

    fn transform(&self) -> Transform {
        const HALF: f32 = 0.5;
        Transform::from_xyz(self.x, self.height*HALF, self.y)
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let config = Config::default();

    let assets = DeerAssets::new(&config);
    let handles = DeerAssetHandles {
        mesh: meshes.add(assets.mesh.clone()),
        material: materials.add(assets.material.clone()),
    };

    commands.insert_resource(assets);
    commands.insert_resource(handles);

    commands.insert_resource(config);

}

#[derive(Component)]
struct State {
    happiness: isize,
    objective: Objective,
}

enum Objective {
    Searching,
    Hunting(Entity),
    Eating(Entity),
    Wandering(Vec3),
}

fn spawn(
    mut commands: Commands,
    mut deers: Query<&mut State>,
    config: Res<Config>,
    asset_handles: Res<DeerAssetHandles>,
) {
    let mut new_deer: Vec<Deer> = Vec::new();
    let deer_count = deers.iter().count();
    if deer_count == 0 {
        let deer = Deer::new(&config);
        new_deer.push(deer);

    } else {
        for mut state in deers.iter_mut() {
            if !(state.happiness >= config.spawn_rate) {
                continue;
            }
            let deer = Deer::new(&config);
            new_deer.push(deer);

            state.happiness = 5; // TODO: enum?
        }
    }
    for d in new_deer.iter() {
        commands.spawn((
            PbrBundle {
                mesh: asset_handles.mesh.clone(),
                material: asset_handles.material.clone(),
                transform: d.transform(),
                ..Default::default()
            },
            State {
                happiness: 5,
                objective: Objective::Searching,
            },
        ));
    }

}



fn deer_ai(
    mut commands: Commands,
    mut q_deer: Query<(Entity, &mut State, &mut Transform, Without<PlantTag>)>,
    q_plant: Query<(Entity, &Transform, With<PlantTag>)>,
    config: Res<Config>,
    time: Res<Time>,
) {
    for (e, mut deer_state, mut deer_transform, _) in q_deer.iter_mut() {
        
        if deer_state.happiness <= 0 {
            if let Some(mut e) = commands.get_entity(e) {
                e.despawn();
            }
        }

        match deer_state.objective {
            Objective::Searching => {
                // Find the closest plant within a certain radius
                let search_radius_squared = config.search_radius * config.search_radius;
                let closest_plant = q_plant.iter()
                    .filter(|(_, plant_transform, _)| {
                        deer_transform.translation.distance_squared(plant_transform.translation) <= search_radius_squared
                    })
                    .min_by_key(|(_, plant_transform, _)| {
                        (deer_transform.translation.distance_squared(plant_transform.translation) * 1000.0) as u32 // Multiplied to maintain precision when converting to integer
                    });

                if let Some((e, _, _)) = closest_plant {
                    deer_state.objective = Objective::Hunting(e);
                } else {
                    let pos = config.get_pos();
                    let dest = Vec3::new(pos.x, 0.0, pos.y);
                    deer_state.objective = Objective::Wandering(dest);
                    deer_state.happiness -= 2;
                    info!("wandering, happiness: {}", deer_state.happiness);
                }
            },
            Objective::Wandering(dest) => {
                // rotate toward if not facing
                let mut direction = dest - deer_transform.translation;
                direction.y = 0.0;
                direction = direction.normalize();
                let forward = deer_transform.forward();
                deer_transform.rotate(Quat::from_rotation_arc(forward, direction));

                if deer_transform.translation.distance(dest) > 2.0 {
                    deer_transform.translation += direction * config.speed * 2.0 * time.delta_seconds();
                } else {
                    deer_state.objective = Objective::Searching;
                    info!("searching");
                }
            }
            Objective::Hunting(target) => {
                if let Some((e, t, _)) = q_plant.iter().find(|(e, _, _)| e == &target) {
                    // rotate toward if not facing
                    let mut direction = t.translation - deer_transform.translation;
                    direction.y = 0.0;
                    direction = direction.normalize();
                    let forward = deer_transform.forward();
                    deer_transform.rotate(Quat::from_rotation_arc(forward, direction));

                    if deer_transform.translation.distance(t.translation) > 1.0 {
                        deer_transform.translation += direction * config.speed * time.delta_seconds();
                    } else {
                        deer_state.objective = Objective::Eating(e);
                        info!("eating");
                    }
                } else {
                    let pos = config.get_pos();
                    let dest = Vec3::new(pos.x, 0.0, pos.y);
                    deer_state.objective = Objective::Wandering(dest);
                    deer_state.happiness -= 2;
                    info!("wandering, happiness: {}", deer_state.happiness);
                };
            },
            Objective::Eating(target) => {
                if let Some(mut e) = commands.get_entity(target) {
                    e.despawn();
                    deer_state.happiness += 1;
                    info!("just ate, happiness: {}", deer_state.happiness);
                    deer_state.objective = Objective::Searching;
                } else {
                    let pos = config.get_pos();
                    let dest = Vec3::new(pos.x, 0.0, pos.y);
                    deer_state.objective = Objective::Wandering(dest);
                    deer_state.happiness -= 2;
                }

            }
        }
    }
}