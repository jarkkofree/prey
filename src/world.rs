use bevy::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
            .add_systems(Startup, setup_world);
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = Mesh::from(shape::Plane {
        size: 50.0,
        ..Default::default()
    });

    let material = StandardMaterial {
        base_color: Color::rgb(0.7, 0.9, 0.3),
        ..Default::default()
    };

    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(material),
        transform: Transform::IDENTITY,
        ..Default::default()
    });

    let light_height = 8.0;
    let light_offset = 4.0;
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            color: Color::WHITE,
            range: 500.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(light_offset, light_height, light_offset),
        ..Default::default()
    });
}