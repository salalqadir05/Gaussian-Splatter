use crate::scene::Scene;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureDimension, TextureFormat, TextureUsages};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::Image;
use glam::Mat4 as GlamMat4;
use std::fs;
use std::io;
use wgpu::Buffer;
use wgpu_types::Extent3d;

#[derive(Component)]
pub struct SplatBuffer {
    pub data: Vec<u8>,
}

pub struct BevyPlugin;

impl Plugin for BevyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, load_splat_file) // Register the system correctly
            .add_systems(Update, render_splats);
    }
}

#[derive(Component)]
pub struct GaussianSplat {
    pub splat_file: String,
    pub transform: Transform,
}

fn setup(mut commands: Commands) {
    commands.spawn((
        GaussianSplat {
            splat_file: "assets/splat_file.splat".to_string(),
            transform: Transform::default(),
        },
        SplatBuffer { data: Vec::new() }, // Initialize SplatBuffer
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
}

#[derive(Debug)]
pub enum FileReading {
    IoError(io::Error),
    InvalidSplatSize,
}

fn load_splat_file(mut query: Query<(&GaussianSplat, &mut SplatBuffer)>, mut scene_query: Query<&mut Scene>) {
    for (gaussian_splat, mut splat_buffer) in query.iter_mut() {
        if splat_buffer.data.is_empty() {
            // Only load if buffer is empty
            match fs::read(&gaussian_splat.splat_file) {
                Ok(raw_data) => {
                    let float_size = std::mem::size_of::<f32>();
                    let splat_size = (3 + 4 + 1 + 2 + 3 + 3 + 16) * float_size;
                    if raw_data.len() % splat_size != 0 {
                        error!("Invalid splat data size for file: {}", gaussian_splat.splat_file);
                        return;
                    }
                    splat_buffer.data = raw_data; // Store raw data in SplatBuffer

                    // Add the line for setting splat_count in the Scene
                    let mut scene = scene_query.single_mut(); // Fetch the scene
                    scene.splat_count = scene.splat_data.len(); // Set splat_count based on the length of splat_data
                }
                Err(e) => {
                    error!("Failed to read splat file {}: {:?}", gaussian_splat.splat_file, e);
                }
            }
        }
    }
}
fn render_splats(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut scene_query: Query<&mut Scene>,
) {
    let mut scene = scene_query.single_mut();

    // Create the sorting buffer for depth sorting (this replaces the old sprite method)
    let sorting_buffer = render_device.create_buffer(&wgpu::BufferDescriptor {
        size: scene.splat_count as u64 * std::mem::size_of::<[u32; 2]>() as u64,
        usage: wgpu::BufferUsages::STORAGE,
        label: Some("Sorting Buffer"),
        mapped_at_creation: false,
    });

    // Store it in the scene
    scene.sorting_buffer = Some(sorting_buffer);

    // Create a texture to render into
    let mut texture = Image::new_fill(
        Extent3d {
            width: 640,
            height: 480,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
    );
    texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;

    // Add the texture to assets
    let texture_handle = images.add(texture);

    let sprite_bundle = SpriteBundle {
        texture: texture_handle,
        ..Default::default()
    };
    commands.spawn(sprite_bundle);
}
