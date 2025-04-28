use bevy::prelude::*;
use bevy::render::camera::Camera;

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_weapon)
           .add_systems(Update, (weapon_controls, update_bullets));
    }
}

#[derive(Component)]
pub struct Weapon {
    pub fire_rate: f32,
    pub last_shot: f32,
    pub ammo: i32,
    pub max_ammo: i32,
    pub _fire_timer: Timer,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            fire_rate: 0.5,
            last_shot: 0.0,
            ammo: 30,
            max_ammo: 30,
            _fire_timer: Timer::from_seconds(0.5, TimerMode::Once),
        }
    }
}

#[derive(Component)]
pub struct Bullet {
    pub speed: f32,
    pub _damage: f32,
    pub direction: Vec3,
}

#[derive(Component)]
pub struct ReloadTimer {
    pub _weapon: Entity,
    pub _duration: Timer,
}

// fn setup_weapon(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
//     // Spawn weapon model
//     commands.spawn((
//         Weapon::default(),
//         PbrBundle {
//             mesh: meshes.add(Mesh::from(shape::Box::new(0.1, 0.1, 0.3))),
//             material: materials.add(StandardMaterial {
//                 base_color: Color::rgb(0.2, 0.2, 0.2),
//                 ..default()
//             }),
//             transform: Transform::from_xyz(0.3, -0.2, -0.5),
//             ..default()
//         },
//     ));
// }


fn setup_weapon(
    mut commands: Commands,
    asset_server: Res<AssetServer>, // No need to pass meshes here
    // mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Load the .glb model
    let pistol_handle = asset_server.load("models/m249_saw_classic.glb");
    // Spawn the weapon (pistol model) using the Scene component
    commands.spawn((
        Weapon::default(),
        SceneBundle {
            scene: pistol_handle.clone(),
            transform: Transform::from_xyz(0.3, -0.2, -0.5).with_scale(Vec3::splat(0.1)),
            ..default()
        },
    ));
}




fn weapon_controls(
    mut commands: Commands,
    time: Res<Time>,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut weapons: Query<(Entity, &mut Weapon)>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Get the first camera and its transform
    if let Some((_camera, camera_transform)) = camera.iter().next() {
        for (entity, mut weapon) in weapons.iter_mut() {
            // Handle reloading
            if keyboard.just_pressed(KeyCode::R) && weapon.ammo < weapon.max_ammo {
                commands.spawn((
                    ReloadTimer {
                        _weapon: entity,
                        _duration: Timer::from_seconds(2.0, TimerMode::Once),
                    },
                ));
            }

            // Handle firing
            if mouse.pressed(MouseButton::Left) && weapon.ammo > 0 && time.elapsed_seconds() - weapon.last_shot >= weapon.fire_rate {
                weapon.last_shot = time.elapsed_seconds();
                weapon.ammo -= 1;

                // Calculate bullet spawn position and direction
                let spawn_pos = camera_transform.translation();
                let direction = camera_transform.forward();

                // Calculate rotation to face the direction of the bullet
                let bullet_rotation = Quat::from_rotation_arc(Vec3::Z, direction); // Rotate to align with the direction

                // Spawn bullet
                commands.spawn((
                    Bullet {
                        speed: 20.0,
                        _damage: 10.0,
                        direction,
                    },
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Capsule {
                            radius: 0.05,
                            depth: 0.25,
                            ..default()
                        })),
                        material: materials.add(StandardMaterial {
                            base_color: Color::rgb(1.0, 0.0, 0.0), // Red color for the bullet
                            ..default()
                        }),
                        transform: Transform::from_translation(spawn_pos).with_rotation(bullet_rotation),
                        ..default()
                    },
                ));

                println!("Weapon fired! Ammo left: {}", weapon.ammo);
            }
        }
    } else {
        // Handle the case when no camera is found
        println!("No camera found in the scene.");
    }
}

fn update_bullets(
    mut commands: Commands,
    time: Res<Time>,
    mut bullets: Query<(Entity, &Bullet, &mut Transform)>,
) {
    for (entity, bullet, mut transform) in bullets.iter_mut() {
        // Move bullet in its direction
        transform.translation += bullet.direction * bullet.speed * time.delta_seconds();

        // Despawn bullet if it goes too far
        if transform.translation.length() > 100.0 {
            commands.entity(entity).despawn();
        }
    }
}
