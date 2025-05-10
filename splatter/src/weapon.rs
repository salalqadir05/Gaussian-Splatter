use bevy::prelude::*;
use bevy::render::camera::Camera;
use crate::player::Player;

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

fn setup_weapon(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<Entity, With<Player>>, // or whatever marker your player has
) {
    let gltf_scene: Handle<Scene> = asset_server.load("models/m249_saw_classic.glb#Scene0");

    if let Ok(player_entity) = query.get_single() {
        commands.entity(player_entity).with_children(|parent| {
            parent.spawn((
                Weapon::default(),
                SceneBundle {
                    scene: gltf_scene,
                    transform: Transform {
                        translation: Vec3::new(0.3, -0.3, 0.6), // Adjust for your model
                        scale: Vec3::splat(0.01),              // Downscale if needed
                        rotation: Quat::IDENTITY,
                    },
                    ..default()
                },
            ));
        });
    }
}

fn weapon_controls(
    mut commands: Commands,
    time: Res<Time>,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut weapons: Query<(Entity, &mut Weapon, &Transform)>, // Include the weapon's transform here
    camera: Query<(&Camera, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Get the first camera and its transform
    if let Some((_camera, _camera_transform)) = camera.iter().next() {
        for (entity, mut weapon, weapon_transform) in weapons.iter_mut() {
            // Handle reloading
            if keyboard.just_pressed(KeyCode::R) && weapon.ammo < weapon.max_ammo {
                commands.spawn((
                    ReloadTimer {
                        _weapon: entity,
                        _duration: Timer::from_seconds(2.0, TimerMode::Once),
                    },
                ));
            }

            // Fire when the left mouse button is pressed
            if mouse.pressed(MouseButton::Left) && weapon.ammo > 0 && time.elapsed_seconds() - weapon.last_shot >= weapon.fire_rate {
                weapon.last_shot = time.elapsed_seconds();
                weapon.ammo -= 1;

                // Spawn the bullet at the weapon's position
                let spawn_pos = weapon_transform.translation;  // The weapon's position
                let direction = weapon_transform.forward();    // The direction the weapon is facing

                // Create the bullet's rotation based on the direction
                let bullet_rotation = Quat::from_rotation_arc(Vec3::Z, direction);

                // Spawn the bullet
                commands.spawn((
                    Bullet {
                        speed: 20.0,
                        _damage: 10.0,
                        direction,
                    },
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.1, sectors: 16, stacks: 8 })),
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
        println!("No camera found in the scene.");
    }
}

fn update_bullets(
    mut commands: Commands,
    time: Res<Time>,
    mut bullets: Query<(Entity, &Bullet, &mut Transform)>,
) {
    for (entity, bullet, mut transform) in bullets.iter_mut() {
        transform.translation += bullet.direction * bullet.speed * time.delta_seconds();
        
        // Despawn the bullet if it goes too far
        if transform.translation.length() > 100.0 {
            commands.entity(entity).despawn();
        }
    }
}
