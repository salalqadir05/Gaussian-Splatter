use crate::player::PlayerPlugin;
use crate::weapon::WeaponPlugin;
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
// use bevy::render::camera::CameraRenderGraph;
// use bevy::render::view::VisibilityBundle;

mod player;
mod weapon;

fn main() {
    App::new()
    // .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Splatter Demo".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((PlayerPlugin, WeaponPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (update_window_title, cursor_grab_system))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _camera_query: Query<Entity, With<Camera>>,
) {
    // Only spawn a camera if one doesn't already exist
    // if camera_query.is_empty() {
    //     commands.spawn((
    //         Camera3dBundle {
    //             camera: Camera {
    //                 order: 0, // Set explicit order
    //                 ..Default::default()
    //             },
    //             transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    //             ..default()
    //         },
    //     ));
    // }

    // Add a light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1000.0,  // Adjust intensity as needed
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-4.0, 5.0, -4.0),  // Change the position
        ..default()
    });


    // Add basic geometry
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.7, 0.6),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });

    // Add floor plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0, subdivisions: 1 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.3, 0.5, 0.3),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
}
fn update_window_title(mut window: Query<&mut Window>, time: Res<Time>) {
    let mut window = window.single_mut();
    window.title = format!("Splatter Demo - {:.0} fps", 1.0 / time.delta_seconds());
}

fn cursor_grab_system(mut window: Query<&mut Window>, mouse: Res<Input<MouseButton>>, key: Res<Input<KeyCode>>) {
    let mut window = window.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
        window.cursor.visible = false;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = bevy::window::CursorGrabMode::None;
        window.cursor.visible = true;
    }
}
