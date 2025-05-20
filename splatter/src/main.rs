use crate::player::PlayerPlugin;
use crate::weapon::WeaponPlugin;
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
use splatter::render_plugin::GaussianSplatRenderPlugin;
use splatter::bevy_plugin::{GaussianSplatPlugin,GaussianSplat,SplatBuffer};
use splatter::component::GaussianSplat as GaussianSplatSplatter;
use bevy::scene::ScenePlugin;
// use bevy::render::RenderApp;
use bevy::core_pipeline::Skybox;
use bevy::prelude::EnvironmentMapLight;
use splatter::component::GaussianSplatBundle;
 use crate::player::Player; 
mod player;
mod weapon;

// fn main() {
//     App::new()
//         .add_plugins(DefaultPlugins.set(WindowPlugin {
//             primary_window: Some(Window {
//                 title: "Splatter Demo".to_string(),
//                 resolution: (1280.0, 720.0).into(),
//                 ..default()
//             }),
//             ..default()
//         }))
//         .add_plugins((PlayerPlugin, WeaponPlugin, GaussianSplatRenderPlugin))
//         .add_systems(Startup, setup)
//         .add_systems(Update, (update_window_title, cursor_grab_system))
//         .run();
// }
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Splatter Demo".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        // .add_plugins((
        //     // ScenePlugin,                    // Required for GLTF scene loading
        //     GaussianSplatPlugin,            // Core splatting engine
        //     GaussianSplatRenderPlugin,      // Your custom depth-aware splat renderer
        //     PlayerPlugin,                   // Handles movement/camera
        //     WeaponPlugin                    // Weapon logic + bullets
        // ))
                .add_plugins((
            // EnvironmentPlugin,
            GaussianSplatPlugin,
            GaussianSplatRenderPlugin,
            PlayerPlugin,
            WeaponPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update,spawn_sky_splats)// Setup scene geometry or lighting
        .add_systems(Update, (
            update_window_title, 
            cursor_grab_system
        ))
        .run();
}

fn generate_sky_splats(resolution: u32) -> Vec<GaussianSplatBundle> {
    let mut splats = Vec::new();
    for i in 0..resolution {
        for j in 0..resolution {
            let theta = (i as f32 / resolution as f32) * std::f32::consts::PI / 2.0; // up to 90°
            let phi = (j as f32 / resolution as f32) * std::f32::consts::TAU;

            let x = theta.sin() * phi.cos();
            let y = theta.cos(); // height
            let z = theta.sin() * phi.sin();

            let pos = Vec3::new(x, y, z) * 500.0; // position the splats far away
            let color = Color::rgba(0.5, 0.7, 1.0, 0.1); // sky blue, semi-transparent

            splats.push(GaussianSplatBundle {
                splat: GaussianSplatSplatter {
                    splat_file: "sky".into(), // placeholder if you use texture lookup
                },
                transform: Transform::from_translation(pos),
                global_transform: GlobalTransform::IDENTITY,
                visibility: Visibility::Visible,
            });
        }
    }
    splats
}
fn spawn_sky_splats(mut commands: Commands) {
    let splats = generate_sky_splats(64); // resolution
    for bundle in splats {
        commands.spawn(bundle);
    }
}
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 1) load your HDR (path is relative to `assets/`)
    // let hdr_handle: Handle<Image> = asset_server.load("models/sky.hdr");
    //   commands.spawn((
    //     Camera3dBundle {
    //         transform: Transform::from_xyz(0.0, 1.7, 5.0)
    //             .looking_at(Vec3::ZERO, Vec3::Y),
    //         camera: Camera { order: 0, ..default() },
    //         ..default()
    //     },
    //     Skybox(hdr_handle.clone()),
    //     EnvironmentMapLight {
    //         diffuse_map: hdr_handle.clone(),
    //         specular_map: hdr_handle.clone(),
    //     },
    //     // your player component:
    //     Player::default(),
    //     InheritedVisibility::VISIBLE,
    //     Visibility::Visible,
    // ))
    // .with_children(|parent| {
    //     // weapon/scene model as child of camera/player
    //     parent.spawn(SceneBundle {
    //         scene: asset_server.load("models/m249_saw_classic.glb#Scene0"),
    //         transform: Transform {
    //             translation: Vec3::new(0.2, -0.2, -0.5),
    //             rotation: Quat::IDENTITY,
    //             scale: Vec3::splat(0.5),
    //         },
    //         ..default()
    //     });
    // });
    let hdr: Handle<Image> = asset_server.load("models/sky.hdr");

    // 2) Build a large UV sphere to act as our dome
    let dome = meshes.add(Mesh::from(shape::UVSphere {
        radius: 100.0,
        sectors: 64,
        stacks: 32,
    }));

    // 3) Create an UNLIT material that simply uses the HDR as its color
    let dome_mat = materials.add(StandardMaterial {
        unlit: true,                              // ignore lighting
        double_sided: true,                       // render inside faces
        base_color_texture: Some(hdr.clone()),    // draw the HDR exactly
        base_color: Color::WHITE,                 // white × texture = original
        // leave emissive/emissive_texture alone
        ..default()
    });
let dome_transform = Transform {
    // first invert the normals by negative scale...
    scale: Vec3::splat(-1.0),
    // then rotate it so the “front” of your HDR lines up:
    rotation: Quat::from_rotation_x(90 as f32), 
    ..default()
};
    // 4) Spawn the dome inverted, so we see the inside
    commands.spawn(PbrBundle {
        mesh: dome,
        material: dome_mat,
        transform: dome_transform,
        ..default()
    });

    // 5) Then spawn your player‑camera, lights, and scene geometry...
    commands.spawn((
        Player::default(),
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 1.7, 5.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
    ));
    

commands.spawn((
    GaussianSplat {
        splat_file: "assets/sky_effect.splat".into(),
        transform: Transform::from_xyz(0.0, 200.0, -500.0),
    },
    SplatBuffer { data: vec![] },
    Transform::from_xyz(0.0, 200.0, -500.0),
    GlobalTransform::default(),
    Visibility::Visible,
    InheritedVisibility::VISIBLE,
    ViewVisibility::default(),
));


    // 2) spawn the environment map light so PBR objects pick up the lighting
    // (existing lights and geometry follow…)

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
            intensity: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-4.0, 5.0, -4.0),
        ..default()
    });

    // basic cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.7, 0.6),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });

    // ground plane
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
