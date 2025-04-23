use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::render::camera::Camera;
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, player_movement)
            .add_systems(Update, player_look)
            .add_systems(Update, sync_scene_camera);
    }
}

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub sensitivity: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 5.0,
            sensitivity: 0.005,
        }
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player::default(),
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 1.7, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera { order: 0, ..default() },
            ..default()
        },
        TransformBundle::from_transform(Transform::from_xyz(0.0, 1.7, 0.0)),
    ));
}

fn player_movement(time: Res<Time>, keyboard: Res<Input<KeyCode>>, mut query: Query<(&Player, &mut Transform)>) {
    for (player, mut transform) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard.pressed(KeyCode::W) {
            direction += transform.forward();
        }
        if keyboard.pressed(KeyCode::S) {
            direction += transform.back();
        }
        if keyboard.pressed(KeyCode::A) {
            direction += transform.left();
        }
        if keyboard.pressed(KeyCode::D) {
            direction += transform.right();
        }
        if keyboard.pressed(KeyCode::Space) {
            direction += Vec3::Y;
        }
        if keyboard.pressed(KeyCode::ShiftLeft) {
            direction += Vec3::NEG_Y;
        }

        if direction != Vec3::ZERO {
            direction = direction.normalize();
            transform.translation += direction * player.speed * time.delta_seconds();
        }
    }
}

fn player_look(mut mouse_motion: EventReader<MouseMotion>, mut query: Query<(&Player, &mut Transform)>) {
    for (player, mut transform) in query.iter_mut() {
        for motion in mouse_motion.read() {
            let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);

            yaw -= motion.delta.x * player.sensitivity;
            pitch -= motion.delta.y * player.sensitivity;
            pitch = pitch.clamp(-1.5, 1.5);

            transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
        }
    }
}
fn sync_scene_camera(mut query: Query<(&mut Transform, &mut Projection), With<Camera3d>>) {
    for (mut transform, mut projection) in query.iter_mut() {
        *transform = Transform::from_xyz(0.0, 1.7, 0.0).looking_at(Vec3::ZERO, Vec3::Y);
        *projection = Projection::Perspective(PerspectiveProjection {
            fov: 45.0_f32.to_radians(),
            aspect_ratio: 640.0 / 480.0,
            near: 0.1,
            far: 100.0,
        });
    }
}
