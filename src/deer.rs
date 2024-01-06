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
    spawn_cost: isize,
    spawn_happiness: isize,
    speed: f32,
    search_radius: f32,
    wander_range: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            color: Color::rgb(0.4, 0.2, 0.1),
            height: 2.0,
            torso_length: 2.0,
            torso_height: 1.0,
            torso_width: 1.0,
            spawn_area: 40.0,
            spawn_rate: 20,
            spawn_cost: 17,
            spawn_happiness: 10,
            speed: 1.0,
            search_radius: 8.0,
            wander_range: 16.0,
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

    fn spawn(t: &Transform, config: &Config) -> Self {
        Deer {
            x: t.translation.x + 1.0,
            y: t.translation.z + 1.0,
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

impl State {
    fn starving(&self) -> bool {
        self.happiness <= 0
    }
    fn happy(&self, config: &Config) -> bool {
        self.happiness >= config.spawn_rate
    }
    fn wander(&mut self, me: &Transform, config: &Config) {
        let pos = config.get_pos();
        let mut dest = Vec3::new(pos.x, 0.0, pos.y);

        if dest.distance(me.translation) > config.wander_range {
            let mut direction = dest - me.translation;
            direction.y = 0.0;
            direction = direction.normalize();
    
            dest = me.translation + direction * config.wander_range;
        }

        self.objective = Objective::Wandering(dest);
        self.happiness -= 2;
    }
    fn eat(&mut self) {
        self.happiness += 1;
        self.objective = Objective::Searching;
    }
    fn face(&self, me: &mut Transform, target: &Transform) -> Vec3 {
        let mut direction = target.translation - me.translation;
        direction.y = 0.0;
        direction = direction.normalize();
        let forward = me.forward();
        if forward.dot(direction) < 0.99 {
            me.rotate(Quat::from_rotation_arc(forward, direction));
        }
        direction
    }
}

enum Objective {
    Searching,
    Hunting(Entity),
    Eating(Entity),
    Wandering(Vec3),
}

fn spawn(
    mut commands: Commands,
    mut deers: Query<(Entity, &Transform, &mut State)>,
    config: Res<Config>,
    asset_handles: Res<DeerAssetHandles>,
) {
    let mut new_deer: Vec<Deer> = Vec::new();
    let deer_count = deers.iter().count();
    if deer_count == 0 {
        let deer = Deer::new(&config);
        new_deer.push(deer);

    } else {
        for (e, t, mut state) in deers.iter_mut() {
            if state.happy(&config) {
                let deer = Deer::spawn(&t, &config);
                new_deer.push(deer);
    
                state.happiness -= config.spawn_cost; // TODO: enum?
            }

            if state.starving() {
                if let Some(mut e) = commands.get_entity(e) {
                    e.despawn();
                }
            }
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
                happiness: config.spawn_happiness,
                objective: Objective::Searching,
            },
        ));
    }

}



fn deer_ai(
    mut commands: Commands,
    mut q_deer: Query<(&mut State, &mut Transform, Without<PlantTag>)>,
    q_plant: Query<(Entity, &Transform, With<PlantTag>)>,
    config: Res<Config>,
    time: Res<Time>,
) {
    for (mut deer_state, mut deer_transform, _) in q_deer.iter_mut() {
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
                    deer_state.wander(&deer_transform, &config);
                }
            },
            Objective::Wandering(dest) => {
                // rotate toward if not facing
                let mut direction = dest - deer_transform.translation;
                direction.y = 0.0;
                direction = direction.normalize();
                let forward = deer_transform.forward();
                deer_transform.rotate(Quat::from_rotation_arc(forward, direction));

                if deer_transform.translation.distance(dest) > 1.5 {
                    deer_transform.translation += direction * config.speed * 2.0 * time.delta_seconds();
                } else {
                    deer_state.objective = Objective::Searching;
                }
            }
            Objective::Hunting(target) => {
                if let Some((e, t, _)) = q_plant.iter().find(|(e, _, _)| e == &target) {
                    // rotate toward if not facing
                    let direction = deer_state.face(&mut deer_transform, &t);

                    if deer_transform.translation.distance(t.translation) > 1.5 {
                        deer_transform.translation += direction * config.speed * time.delta_seconds();
                    } else {
                        deer_state.objective = Objective::Eating(e);
                    }
                } else {
                    deer_state.wander(&deer_transform, &config);
                };
            },
            Objective::Eating(target) => {
                if let Some(mut e) = commands.get_entity(target) {
                    e.despawn();
                    deer_state.eat();
                } else {
                    deer_state.wander(&deer_transform, &config);
                }

            }
        }
    }
}