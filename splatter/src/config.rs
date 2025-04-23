// src/config.rs
use wgpu::SurfaceConfiguration;
// use wgpu::{CompositeAlphaMode, PresentMode, SurfaceConfiguration, TextureFormat, TextureUsages};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthSorting {
    Cpu,
    Gpu,
    GpuIndirectDraw,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub surface_configuration: wgpu::SurfaceConfiguration, // ← Add this
    pub depth_sorting: DepthSorting,
    pub use_covariance_for_scale: bool,
    pub use_unaligned_rectangles: bool,
    pub spherical_harmonics_order: u32,
    pub max_splat_count: u32,
    pub radix_bits_per_digit: u32,
    pub frustum_culling_tolerance: f32,
    pub ellipse_margin: f32,
    pub splat_scale: f32,
}
