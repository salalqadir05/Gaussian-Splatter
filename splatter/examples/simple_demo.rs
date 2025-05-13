use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use splatter::render_plugin::GaussianSplatRenderPlugin;
use splatter::scene::ScenePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            GaussianSplatRenderPlugin,
            ScenePlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 1.6, 5.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        // Use the camera driver graph
        CameraRenderGraph::new(bevy::render::main_graph::node::CAMERA_DRIVER),
    ));

    // Lighting
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Add some polygonal geometry that will be rendered in front of the splats
    // when closer to the camera
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(2.0, 2.0, 2.0))),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.2, 0.2),
            ..default()
        }),
        transform: Transform::from_xyz(3.0, 1.0, 0.0),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(2.0, 2.0, 2.0))),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.2, 0.8, 0.2),
            ..default()
        }),
        transform: Transform::from_xyz(-3.0, 1.0, 0.0),
        ..default()
    });

    // Add a floor
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0, subdivisions: 1 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.3, 0.3, 0.3),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
} 