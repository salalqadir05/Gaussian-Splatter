use crate::scene::Scene;
use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use wgpu::{BindGroupLayout, Buffer, CommandEncoder, ComputePipeline, Device, Queue, RenderPass, RenderPassDescriptor, RenderPipeline, TextureView};
//use crate::config::{Config, DepthSorting}; // Assuming you have a Config struct
use crate::config::{Config, DepthSorting};
use std::borrow::Cow;
use bevy::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    projection: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    params: [f32; 4], // z_near, z_far, frustum_culling_tolerance, ellipse_margin
    counts: [f32; 4], // splat_scale, splat_count, visible_entries.len(), padding
}

#[derive(Resource)]
pub struct Renderer {
    pub pipeline: Option<RenderPipeline>,
    pub bind_group_layout: Option<BindGroupLayout>,
    pub vertex_buffer: Option<Buffer>,
    pub config: Config,
    pub uniform_buffer: Option<Buffer>,
}

impl Renderer {
    pub fn new(device: &Device, config: Config) -> Self {
        Self {
            pipeline: None,
            bind_group_layout: None,
            vertex_buffer: None,
            config,
            uniform_buffer: None,
        }
    }

    pub fn initialize(&mut self, device: &Device) -> Result<(), wgpu::Error> {
        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Gaussian Splat Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Gaussian Splat Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Gaussian Splat Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders.wgsl"))),
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Gaussian Splat Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.config.surface_configuration.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.pipeline = Some(pipeline);
        self.bind_group_layout = Some(bind_group_layout);
        self.uniform_buffer = Some(uniform_buffer);

        Ok(())
    }

    pub fn render(&self, encoder: &mut CommandEncoder, view: &TextureView, queue: &Queue, scene: &mut Scene) {
        if let Some(pipeline) = &self.pipeline {
            if let Some(uniform_buffer) = &self.uniform_buffer {
                // Update uniforms
                let uniforms = Uniforms {
                    projection: scene.camera.projection.to_cols_array_2d(),
                    view: scene.camera.view.to_cols_array_2d(),
                    params: [
                        scene.camera.z_near,
                        scene.camera.z_far,
                        self.config.frustum_culling_tolerance,
                        self.config.ellipse_margin,
                    ],
                    counts: [
                        self.config.splat_scale,
                        scene.splat_count as f32,
                        0.0,
                        0.0,
                    ],
                };

                queue.write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

                // Create render pass
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Gaussian Splat Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(pipeline);
                render_pass.draw(0..scene.splat_count as u32, 0..1);
            }
        }
    }

    pub fn cleanup(&mut self) {
        if let Some(buffer) = self.vertex_buffer.take() {
            buffer.destroy();
        }
        if let Some(buffer) = self.uniform_buffer.take() {
            buffer.destroy();
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SplatEntry {
    pub center: [f32; 3],
    pub color: [f32; 4],
    pub depth: f32,
    pub scale: [f32; 2],
    pub normal: [f32; 3],
    pub padding: f32,
    pub ellipse_basis: [f32; 3],
    pub padding2: f32,
}

impl From<&crate::scene::Splat> for SplatEntry {
    fn from(splat: &crate::scene::Splat) -> Self {
        SplatEntry {
            center: splat.center,
            color: splat.color,
            depth: splat.depth,
            scale: splat.scale,
            normal: splat.normal,
            padding: 0.0,
            ellipse_basis: splat.ellipse_basis,
            padding2: 0.0,
        }
    }
}
