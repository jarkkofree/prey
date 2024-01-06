use bevy::{
    prelude::*,
    input::mouse::MouseMotion,
    window::CursorGrabMode, time,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(Update, (look, walk));
    }
}

fn setup(
    mut commands: Commands,
) {
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(20.0, 3.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

const LOOK_SENSITIVITY: f32 = 0.1;
use std::f64::consts::PI;
const UP: f32 = PI as f32 / 2.0;

fn look(
    mut q: Query<&mut Transform, With<Camera>>,
    mut er_mm: EventReader<MouseMotion>,
    mut windows: Query<&mut Window>,
    mouse: Res<Input<MouseButton>>,
    time: Res<time::Time>,
) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    if mouse.just_pressed(MouseButton::Right) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }

    if window.cursor.grab_mode == CursorGrabMode::Locked {
        for ev in er_mm.read() {
            for mut t in q.iter_mut() {
                let delta_x = -ev.delta.x * LOOK_SENSITIVITY * time.delta_seconds();
                let delta_y = -ev.delta.y * LOOK_SENSITIVITY * time.delta_seconds();

                // Calculate the potential new rotation
                let pitch = Quat::from_rotation_x(delta_y);
                let pitch_rotation = t.rotation * pitch;

                // Calculate the camera's new up vector
                let new_up = pitch_rotation.mul_vec3(Vec3::Y);

                // Check the angle with global up
                let angle_with_up = Vec3::Y.angle_between(new_up);

                // Apply the rotation if within bounds
                if angle_with_up < UP && angle_with_up > -UP {
                    t.rotation = pitch_rotation;
                }

                // Apply yaw rotation globally (left/right)
                let yaw = Quat::from_rotation_y(delta_x);
                t.rotation = yaw * t.rotation;
            }
        }
    }
}

const SPEED: f32 = 10.0;

fn walk(
    mut q: Query<&mut Transform, With<Camera>>,
    keys: ResMut<Input<KeyCode>>,
    time: Res<Time>,
) {
    for mut transform in q.iter_mut() {

        let mut movement = Vec3::ZERO;

        if keys.pressed(KeyCode::W) {
            movement += transform.forward();
        }
        if keys.pressed(KeyCode::S) {
            movement += transform.back();
        }
        if keys.pressed(KeyCode::A) {
            movement += transform.left();
        }
        if keys.pressed(KeyCode::D) {
            movement += transform.right();
        }

        if movement.length_squared() > 0.0 {
            movement.y = 0.0;
            transform.translation += movement.normalize() * SPEED * time.delta_seconds();
        }

    }
}