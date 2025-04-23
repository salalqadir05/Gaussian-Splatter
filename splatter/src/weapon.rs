use bevy::prelude::*;

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, setup_weapon).add_systems(Update, weapon_controls);
    }
}

#[derive(Component)]
pub struct Weapon {
    pub fire_rate: f32,
    pub last_shot: f32,
    pub ammo: i32,
    pub max_ammo: i32,
    pub fire_timer: Timer, // Added timer for fire rate control
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            fire_rate: 0.5, // Default fire rate (seconds between shots)
            last_shot: 0.0,
            ammo: 30,
            max_ammo: 30,
            fire_timer: Timer::from_seconds(0.5, TimerMode::Once), // Initialize fire timer
        }
    }
}

#[derive(Component)]
pub struct Bullet {
    pub speed: f32,
    pub damage: f32,
}

#[derive(Component)]
pub struct ReloadTimer {
    pub weapon: Entity,
    pub duration: Timer,
}

fn setup_weapon(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load weapon texture and spawn it in the world
    let texture_handle = asset_server.load("weapon.png");
    commands.spawn((
        Weapon {
            fire_rate: 0.5,
            last_shot: 0.0,
            ammo: 30,
            max_ammo: 30,
            fire_timer: Timer::from_seconds(0.5, TimerMode::Once),
        },
        SpriteBundle {
            texture: texture_handle,
            transform: Transform::from_xyz(0.3, -0.3, -0.1),
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
) {
    for (entity, mut weapon) in weapons.iter_mut() {
        // Handle reloading
        if keyboard.just_pressed(KeyCode::R) && weapon.ammo < weapon.max_ammo {
            // Start the reload timer
            commands.spawn((ReloadTimer {
                weapon: entity,
                duration: Timer::from_seconds(2.0, TimerMode::Once),
            },));
        }

        // Handle firing
        if mouse.pressed(MouseButton::Left) && weapon.ammo > 0 && time.elapsed_seconds() - weapon.last_shot >= weapon.fire_rate {
            // Update last shot time and decrement ammo
            weapon.last_shot = time.elapsed_seconds();
            weapon.ammo -= 1;

            // Spawn bullet (could include the position and movement logic)
            commands.spawn((
                Bullet { speed: 20.0, damage: 10.0 },
                TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
            ));

            println!("Weapon fired! Ammo left: {}", weapon.ammo);
        }
    }
}
