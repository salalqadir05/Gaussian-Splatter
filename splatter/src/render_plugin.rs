// use crate::config::{Config, DepthSorting};
// use wgpu::SurfaceConfiguration;

use crate::scene::Scene;
use bevy::asset::Handle;
use bevy::prelude::*;
use bevy::render::{
    render_phase::{DrawFunctionId, PhaseItem},
    render_resource::*,
    renderer::{RenderDevice, RenderQueue},
    view::ViewTarget,
    ExtractSchedule, RenderApp,
};
use bevy::utils::nonmax::NonMaxU32;
use std::borrow::Cow;
use std::default::Default;
use wgpu::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthSorting {
    Cpu,
    Gpu,
    GpuIndirectDraw,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub surface_configuration: wgpu::SurfaceConfiguration,
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

#[derive(Resource)]
pub struct Renderer {
    pub config: Config,
    pub pipeline: Option<RenderPipeline>,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
struct ExtractSplatSet;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct GaussianSplatRenderPlugin;

impl Plugin for GaussianSplatRenderPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<Renderer>()
            .add_systems(ExtractSchedule, extract_splats.in_set(ExtractSplatSet));
    }
}

pub struct PipelineResource {
    pub pipeline: RenderPipeline,
}

pub struct GaussianSplatPhase {
    pub entity: Entity,
    pub pipeline: PipelineResource,
    pub distance: f32,
    pub draw_function: DrawFunctionId,
    pub instance_index: usize,
}

impl Clone for GaussianSplatPhase {
    fn clone(&self) -> Self {
        Self {
            entity: self.entity,
            pipeline: PipelineResource {
                pipeline: self.pipeline.pipeline.clone(),
            },
            distance: self.distance,
            draw_function: self.draw_function,
            instance_index: self.instance_index,
        }
    }
}

impl PhaseItem for GaussianSplatPhase {
    type SortKey = u64;

    fn entity(&self) -> Entity {
        self.entity
    }

    fn sort_key(&self) -> Self::SortKey {
        0
    }

    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }

    fn sort(items: &mut [Self]) {
        items.sort_by_key(|item| item.sort_key());
    }

    fn batch_range(&self) -> &std::ops::Range<u32> {
        static RANGE: std::ops::Range<u32> = 0..0;
        &RANGE
    }

    fn batch_range_mut(&mut self) -> &mut std::ops::Range<u32> {
        panic!("batch_range_mut() should not be called directly!")
    }

    fn dynamic_offset(&self) -> Option<NonMaxU32> {
        None
    }

    fn dynamic_offset_mut(&mut self) -> &mut Option<NonMaxU32> {
        panic!("dynamic_offset_mut() should not be called directly!")
    }
}

impl FromWorld for Renderer {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let config = Config {
            surface_configuration: wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: 800,
                height: 600,
                present_mode: wgpu::PresentMode::Immediate,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
            },
            depth_sorting: DepthSorting::Cpu,
            use_covariance_for_scale: false,
            use_unaligned_rectangles: false,
            spherical_harmonics_order: 0,
            max_splat_count: 10_000,
            radix_bits_per_digit: 1,
            frustum_culling_tolerance: 0.1,
            ellipse_margin: 0.01,
            splat_scale: 1.0,
        };

        let renderer = Renderer::new(&render_device.clone(), config.clone());

        let sc = &config.surface_configuration;
        let _depth_texture = render_device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: sc.width,
                height: sc.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        renderer
    }
}

impl Renderer {
    pub fn new(_render_device: &RenderDevice, config: Config) -> Self {
        Self {
            config,
            pipeline: None,
        }
    }

    pub fn render_scene(
        &mut self,
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
        view_target: &ViewTarget,
        shaders: Handle<Shader>,
    ) {
        let shader_handle = Handle::<Shader>::weak_from_u128(123456789);
        if !shaders.ge(&shader_handle) {
            return;
        }

        let _shader = shader_handle.to_owned();

        if self.pipeline.is_none() {
            let bind_group_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Gaussian Splat Bind Group Layout"),
                entries: &[],
            });

            let pipeline_layout = render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Gaussian Splat Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

            let vertex_shader_module = render_device.create_shader_module(ShaderModuleDescriptor {
                label: Some("Vertex Shader"),
                source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders.wgsl"))),
            });

            let fragment_shader_module = render_device.create_shader_module(ShaderModuleDescriptor {
                label: Some("Fragment Shader"),
                source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders.wgsl"))),
            });

            let render_pipeline_descriptor = wgpu::RenderPipelineDescriptor {
                label: Some("Gaussian Splat Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vertex_shader_module,
                    entry_point: "main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fragment_shader_module,
                    entry_point: "main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::PointList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
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
            };

            self.pipeline = Some(render_device.create_render_pipeline(&render_pipeline_descriptor));
        }

        let texture = view_target.main_texture();
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        if let Some(pipeline) = &self.pipeline {
            let mut encoder = render_device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Gaussian Splat Encoder"),
            });

            {
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Splat Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &texture_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(pipeline);
                render_pass.draw(0..1, 0..1);
            }

            render_queue.submit(std::iter::once(encoder.finish()));
        }
    }
}

fn extract_splats(_commands: Commands, _scene: Res<Scene>) {
    // Extract splats from the scene
    // This is a placeholder for the actual extraction logic
}
