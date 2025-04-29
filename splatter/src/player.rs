use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::input::keyboard::KeyCode;
use bevy::render::view::visibility::{InheritedVisibility, Visibility};
struct CameraController {
    _speed: f32,
    _rotation_speed: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        CameraController {
            _speed: 5.0,
            _rotation_speed: 1.0,
        }
    }
}
pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, spawn_player)
            .add_systems(Update, (player_movement, player_look))
        .add_systems(Update, fire_bullet);
    }
}

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub sensitivity: f32,
    pub jump_force: f32,
    pub gravity: f32,
    pub velocity: Vec3,
    pub is_grounded: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 5.0,
            sensitivity: 0.005,
            jump_force: 5.0,
            gravity: -9.81,
            velocity: Vec3::ZERO,
            is_grounded: true,
        }
    }
}
#[derive(Component)]
// In components.rs
pub struct Velocity(pub Vec3);
#[derive(Component)]

pub struct  Bullet;
fn fire_bullet(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    query: Query<&Transform, With<Player>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Ok(player_transform) = query.get_single() {
            // Spawn bullet in the direction the camera is looking
            let forward = player_transform.forward();

            commands.spawn((
                Bullet,
                PbrBundle {
                    transform: Transform {
                        translation: player_transform.translation + forward * 1.0,
                        ..default()
                    },
                    ..default()
                },
                Velocity(forward * 20.0), // Custom component to handle movement
            ));
        }
    }
}


fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn((
            Player::default(),
            InheritedVisibility::VISIBLE,
            Visibility::Visible,
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 1.7, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
                camera: Camera { order: 0, ..default() },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneBundle {
                    scene: asset_server.load("models/m249_saw_classic.glb#Scene0"),
                    transform: Transform {
                        translation: Vec3::new(0.2, -0.2, -0.5),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::splat(0.5),
                    },
                    ..default()
                },
                // InheritedVisibility::VISIBLE,
                // Visibility::Visible,
            ));
        });
}

// fn spawn_player(mut commands: Commands) {
//     commands.spawn((
//         Player::default(),
//         Camera3dBundle {
//             transform: Transform::from_xyz(0.0, 1.7, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
//             camera: Camera { order: 0, ..default() },
//             ..default()
//         },
//         CameraController::default(),
//     ));
// }

// // A system to control the camera position dynamically based on user input
// fn camera_movement(
//     mut query: Query<(&mut Transform, &CameraController)>,
//     keyboard_input: Res<Input<KeyCode>>,
//     time: Res<Time>,
// ) {
//     for (mut transform, controller) in query.iter_mut() {
//         let mut movement = Vec3::ZERO;

//         // Moving the camera in different directions using the arrow keys or WASD
//         if keyboard_input.pressed(KeyCode::W) {
//             movement.z -= controller.speed * time.delta_seconds();
//         }
//         if keyboard_input.pressed(KeyCode::S) {
//             movement.z += controller.speed * time.delta_seconds();
//         }
//         if keyboard_input.pressed(KeyCode::A) {
//             movement.x -= controller.speed * time.delta_seconds();
//         }
//         if keyboard_input.pressed(KeyCode::D) {
//             movement.x += controller.speed * time.delta_seconds();
//         }

//         // Apply the movement
//         transform.translation += movement;

//         // Optionally, add camera rotation control (left/right rotation)
//         if keyboard_input.pressed(KeyCode::Left) {
//             transform.rotate(Quat::from_rotation_y(controller.rotation_speed * time.delta_seconds()));
//         }
//         if keyboard_input.pressed(KeyCode::Right) {
//             transform.rotate(Quat::from_rotation_y(-controller.rotation_speed * time.delta_seconds()));
//         }
//     }
// }

fn player_movement(
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    for (mut player, mut transform) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        // Get forward and right vectors from camera rotation
        let forward = transform.forward();
        let right = transform.right();

        // Movement input
        if keyboard.pressed(KeyCode::W) {
            direction += forward;
        }
        if keyboard.pressed(KeyCode::S) {
            direction -= forward;
        }
        if keyboard.pressed(KeyCode::A) {
            direction -= right;
        }
        if keyboard.pressed(KeyCode::D) {
            direction += right;
        }

        // Normalize movement direction
        if direction != Vec3::ZERO {
            direction = direction.normalize();
        }

        // Apply movement
        let movement = direction * player.speed * time.delta_seconds();
        transform.translation += movement;

        // Jump
        if keyboard.just_pressed(KeyCode::Space) && player.is_grounded {
            player.velocity.y = player.jump_force;
            player.is_grounded = false;
        }

        // Apply gravity
        if !player.is_grounded {
            player.velocity.y += player.gravity * time.delta_seconds();
            transform.translation.y += player.velocity.y * time.delta_seconds();

            // Ground check
            if transform.translation.y <= 1.7 {
                transform.translation.y = 1.7;
                player.velocity.y = 0.0;
                player.is_grounded = true;
            }
        }
    }
}

fn player_look(
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&Player, &mut Transform)>,
) {
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
