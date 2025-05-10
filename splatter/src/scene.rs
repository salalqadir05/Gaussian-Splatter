use bevy::prelude::*;
use bevy::render::render_resource::Buffer as BevyBuffer;
use bevy::render::renderer::RenderDevice;
// use bevy::render::texture::Image;
use bytemuck;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};
use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use ply_rs::parser::Parser;
use ply_rs::ply::DefaultElement;
pub struct ScenePlugin;
use wgpu::util::BufferInitDescriptor;
// use wgpu::Buffer as WgpuBuffer;
#[repr(C)] // ensure C-compatible field ordering & alignment
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ShaderSplat {
    pub rotation: [f32; 4], // 16 bytes
    pub center: [f32; 3],   // 12 bytes
    pub _pad0: f32,         // pad to 16-byte boundary

    pub scale: [f32; 2], //  8 bytes
    pub alpha: f32,      //  4 bytes
    pub _pad1: [f32; 3], // pad to 16-byte boundary

    // e.g. spherical-harmonic color coefficients
    pub color_sh: [f32; 48], // 192 bytes
}
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Scene>()
            .add_systems(Startup, setup_scene)
            .add_systems(Update, convert_splat_data);
    }
}

#[derive(Component)]
pub struct GaussianBackground;

pub struct Camera {
    pub projection: Mat4,
    pub view: Mat4,
    pub z_near: f32,
    pub z_far: f32,
}
impl Camera {
    pub fn get_clip_space_position(&self, position: &Vec3) -> Vec3 {
        let view_pos = self.view * Vec4::new(position.x, position.y, position.z, 1.0);
        let clip_pos = self.projection * view_pos;
        clip_pos.truncate() // clip_pos.w
    }
}

#[derive(Component)]
pub struct Splat {
    pub model_matrix: Mat4,
    pub center: [f32; 3],
    pub color: [f32; 4],
    pub depth: f32,
    pub scale: [f32; 2],
    pub normal: [f32; 3],
    pub ellipse_basis: [f32; 3],
}

#[derive(Component, Resource)]
pub struct Scene {
    pub splat_count: usize,     // Change from u32 to usize
    pub splat_data: Vec<Splat>, // Change from Vec<u8> to Vec<Splat>
    pub splat_positions: Vec<[f32; 3]>,
    pub compute_bind_groups: Vec<wgpu::BindGroup>,
    pub render_bind_group: Option<wgpu::BindGroup>,
    pub splat_buffer: Option<BevyBuffer>, // ← NEW
    pub camera: Camera,
    pub sorting_buffer: Option<BevyBuffer>,
}
impl Scene {
    pub fn load_chunk(
        &mut self,
        queue: &mut wgpu::Queue,
        file: &mut File,
        header_size: u16,
        range: std::ops::Range<usize>,
    ) {
        file.seek(std::io::SeekFrom::Start(header_size as u64)).expect("Failed to seek file");

        let chunk_size = range.end - range.start;
        let mut buffer = vec![0u8; chunk_size]; 

        file.read_exact(&mut buffer).expect("Failed to read chunk data");

        self.process_chunk_data(queue, &buffer);

    }

    fn process_chunk_data(&mut self, _queue: &mut wgpu::Queue, _chunk_data: &[u8]) {
        // Logic to process chunk data, like updating buffers, etc.
        // This is just a placeholder; actual implementation will depend on the scene's requirements.
        // Example:
        // self.buffer.write(&chunk_data, queue);
    }
    pub fn parse_file_header(mut file: File) -> (u16, usize, File) {
        let mut buffer = [0u8; 2]; // for u16
        file.read_exact(&mut buffer).expect("Failed to read file header size");
        let file_header_size = u16::from_le_bytes(buffer);

        let mut buffer = [0u8; 8]; // for usize (assuming 64-bit)
        file.read_exact(&mut buffer).expect("Failed to read splat count");
        let splat_count = usize::from_le_bytes(buffer);

        // Seek to the start of the splats (after the header)
        file.seek(SeekFrom::Start(file_header_size as u64)).expect("Failed to seek to splat data");

        (file_header_size, splat_count, file)
    }
    pub fn new() -> Self {
        Self {
            splat_count: 0,
            splat_data: Vec::new(),
            splat_positions: Vec::new(),
            compute_bind_groups: Vec::new(),
            render_bind_group: None,
            splat_buffer: None,
            sorting_buffer: None,
            camera: Camera {
                projection: Mat4::perspective_rh_gl(45.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0),
                view: Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y),
                z_near: 0.1,
                z_far: 100.0,
            },
        }
    }
    pub fn load_splat_file(&mut self, path: &str) {
        let raw_data = fs::read(path).expect("Failed to read splat file");
        let float_size = std::mem::size_of::<f32>();
        let splat_size = (3 + 4 + 1 + 2 + 3 + 3 + 16) * float_size; // Size of one Splat
        assert_eq!(raw_data.len() % splat_size, 0, "Invalid splat data size");

        self.splat_data = raw_data
            .chunks(splat_size)
            .map(|chunk| {
                let floats = bytemuck::cast_slice::<u8, f32>(chunk);
                let mut offset = 0;

                let center = [floats[offset], floats[offset + 1], floats[offset + 2]];
                offset += 3;

                let color = [floats[offset], floats[offset + 1], floats[offset + 2], floats[offset + 3]];
                offset += 4;

                let depth = floats[offset];
                offset += 1;

                let scale = [floats[offset], floats[offset + 1]];
                offset += 2;

                let normal = [floats[offset], floats[offset + 1], floats[offset + 2]];
                offset += 3;

                let ellipse_basis = [floats[offset], floats[offset + 1], floats[offset + 2]];
                offset += 3;

                let model_matrix = Mat4::from_cols_array(&[
                    floats[offset],
                    floats[offset + 1],
                    floats[offset + 2],
                    floats[offset + 3],
                    floats[offset + 4],
                    floats[offset + 5],
                    floats[offset + 6],
                    floats[offset + 7],
                    floats[offset + 8],
                    floats[offset + 9],
                    floats[offset + 10],
                    floats[offset + 11],
                    floats[offset + 12],
                    floats[offset + 13],
                    floats[offset + 14],
                    floats[offset + 15],
                ]);

                Splat {
                    model_matrix,
                    center,
                    color,
                    depth,
                    scale,
                    normal,
                    ellipse_basis,
                }
            })
            .collect();

        self.splat_count = self.splat_data.len(); // Remove as u32 cast, use usize
        println!("Loaded {} splats from PLY", self.splat_data.len());
    }
    pub fn load_splats_from_ply(&mut self, path: &str) {
        let mut f = std::fs::File::open(path).expect("Failed to open PLY file");
        let mut reader = std::io::BufReader::new(&mut f);
        let parser = Parser::<DefaultElement>::new();
        let ply = parser.read_ply(&mut reader).expect("Failed to parse PLY");
        let vertex_list = ply.payload.get("vertex").expect("No vertex element in PLY");
        self.splat_data.clear();
        for v in vertex_list {
            let x = v.get("x").and_then(|p| match p {
                ply_rs::ply::Property::Float(f) => Some(*f as f32),
                ply_rs::ply::Property::Double(d) => Some(*d as f32),
                ply_rs::ply::Property::Int(i) => Some(*i as f32),
                ply_rs::ply::Property::UInt(u) => Some(*u as f32),
                _ => None,
            }).unwrap_or(0.0);

            let y = v.get("y").and_then(|p| match p {
                ply_rs::ply::Property::Float(f) => Some(*f as f32),
                ply_rs::ply::Property::Double(d) => Some(*d as f32),
                ply_rs::ply::Property::Int(i) => Some(*i as f32),
                ply_rs::ply::Property::UInt(u) => Some(*u as f32),
                _ => None,
            }).unwrap_or(0.0);

            let z = v.get("z").and_then(|p| match p {
                ply_rs::ply::Property::Float(f) => Some(*f as f32),
                ply_rs::ply::Property::Double(d) => Some(*d as f32),
                ply_rs::ply::Property::Int(i) => Some(*i as f32),
                ply_rs::ply::Property::UInt(u) => Some(*u as f32),
                _ => None,
            }).unwrap_or(0.0);

            let r = v.get("red").and_then(|p| match p {
                ply_rs::ply::Property::UChar(u) => Some(*u as f32 / 255.0),
                ply_rs::ply::Property::Char(i) => Some(*i as f32 / 255.0),
                ply_rs::ply::Property::Int(i) => Some(*i as f32 / 255.0),
                ply_rs::ply::Property::UInt(u) => Some(*u as f32 / 255.0),
                _ => None,
            }).unwrap_or(1.0);

            let g = v.get("green").and_then(|p| match p {
                ply_rs::ply::Property::UChar(u) => Some(*u as f32 / 255.0),
                ply_rs::ply::Property::Char(i) => Some(*i as f32 / 255.0),
                ply_rs::ply::Property::Int(i) => Some(*i as f32 / 255.0),
                ply_rs::ply::Property::UInt(u) => Some(*u as f32 / 255.0),
                _ => None,
            }).unwrap_or(1.0);

            let b = v.get("blue").and_then(|p| match p {
                ply_rs::ply::Property::UChar(u) => Some(*u as f32 / 255.0),
                ply_rs::ply::Property::Char(i) => Some(*i as f32 / 255.0),
                ply_rs::ply::Property::Int(i) => Some(*i as f32 / 255.0),
                ply_rs::ply::Property::UInt(u) => Some(*u as f32 / 255.0),
                _ => None,
            }).unwrap_or(1.0);

            let a = v.get("alpha").and_then(|p| match p {
                ply_rs::ply::Property::UChar(u) => Some(*u as f32 / 255.0),
                ply_rs::ply::Property::Char(i) => Some(*i as f32 / 255.0),
                ply_rs::ply::Property::Int(i) => Some(*i as f32 / 255.0),
                ply_rs::ply::Property::UInt(u) => Some(*u as f32 / 255.0),
                _ => None,
            }).unwrap_or(1.0);

            self.splat_data.push(Splat {
                model_matrix: Mat4::IDENTITY,
                center: [x, y, z],
                color: [r, g, b, a],
                depth: 0.0,
                scale: [0.05, 0.05], // Default scale for splats
                normal: [0.0, 1.0, 0.0], // Default normal
                ellipse_basis: [1.0, 0.0, 0.0], // Default basis
            });
        }
        self.splat_count = self.splat_data.len();
        println!("Loaded {} splats from PLY", self.splat_data.len());
    }

    // pub fn render(&mut self, render_device: Res<RenderDevice>, render_queue: Res<RenderQueue>, texture: &Image) {
    //     // Placeholder for rendering implementation
    // }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

fn setup_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Spawn the scene component
    let mut scene = Scene::default();
    scene.load_splats_from_ply("assets/models/test.ply");
    commands.spawn((scene, SpatialBundle::default()));

    // Create a simple room
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Box::new(10.0, 5.0, 10.0).into()),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.8, 0.8),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 2.5, 0.0),
        ..default()
    });

    // Add some props
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Box::new(1.0, 1.0, 1.0).into()),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.4, 0.4, 0.8),
            ..default()
        }),
        transform: Transform::from_xyz(2.0, 0.5, 2.0),
        ..default()
    });
}
fn _setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 5.0 })),
        material: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
        transform: Transform::from_xyz(0.0, 2.5, 0.0),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.5, 0.5, 0.7).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
}

fn convert_splat_data(mut scene: ResMut<Scene>, render_device: Res<RenderDevice>) {
    let shader_splats: Vec<ShaderSplat> = scene
        .splat_data
        .iter()
        .map(|splat| {
            ShaderSplat {
                rotation: [0.0, 0.0, 0.0, 1.0], // Placeholder
                center: splat.center,
                _pad0: 0.0,
                scale: splat.scale,
                alpha: splat.color[3],
                color_sh: [0.0; 48],
                _pad1: [0.0; 3],
            }
        })
        .collect();
    let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("Splat Buffer"),
        contents: bytemuck::cast_slice(&shader_splats),
        usage: wgpu::BufferUsages::STORAGE,
    });
    scene.splat_buffer = Some(buffer);
}
