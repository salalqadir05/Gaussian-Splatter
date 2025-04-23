use crate::scene::Scene;
use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use wgpu::{BindGroupLayout, Buffer, CommandEncoder, ComputePipeline, Device, Queue, RenderPass, RenderPassDescriptor, RenderPipeline, TextureView};
//use crate::config::{Config, DepthSorting}; // Assuming you have a Config struct
use crate::config::{Config, DepthSorting};
use std::borrow::Cow;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    projection: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    params: [f32; 4], // z_near, z_far, frustum_culling_tolerance, ellipse_margin
    counts: [f32; 4], // splat_scale, splat_count, visible_entries.len(), padding
}
pub struct Renderer {
    pub sorting_bind_group_layout: wgpu::BindGroupLayout,
    pub compute_pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: RenderPipeline,
    pub bind_group_layout: BindGroupLayout,
    pub vertex_buffer: Buffer,

    pub config: Config, // Add this line

    pub entry_buffer_a: Buffer,     // Add this line
    pub uniform_buffer: Buffer,     // Add this line
    pub sorting_buffer: Buffer,     // Add this line
    pub sorting_buffer_size: usize, // Add this line

    pub radix_sort_a_pipeline: ComputePipeline,
    pub workgroup_entries_a: u32,

    pub radix_sort_b_pipeline: ComputePipeline,
    pub radix_digit_places: u32,

    pub radix_base: u32,
    pub max_tile_count_c: u32,

    pub radix_sort_c_pipeline: ComputePipeline,
    pub workgroup_entries_c: u32,
}

impl Renderer {
    pub fn new(device: &Device, surface_config: &wgpu::SurfaceConfiguration, config: Config) -> Result<Self, wgpu::Error> {
        // ... (Your existing initialization code) ...
        let sorting_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Sorting Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&sorting_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Example initialization for the new fields.  Adjust as needed.
        let entry_buffer_a = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("entry_buffer_a"),
            size: 1024, // Initial size, adjust as needed
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform_buffer"),
            size: 256, // Adjust as needed
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sorting_buffer_size = 1024 * 1024; //Adjust
        let sorting_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sorting_buffer"),
            size: sorting_buffer_size as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::INDIRECT,
            mapped_at_creation: false,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/radix_sort_a.wgsl").into()),
        });

        let radix_sort_a_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Radix Sort A Pipeline"),
            layout: None,
            module: &shader,
            entry_point: "main",
        });

        let workgroup_entries_a = 64; // Example, adjust as needed

        let shader_b = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/radix_sort_b.wgsl").into()),
        });

        let radix_sort_b_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Radix Sort B Pipeline"),
            layout: None,
            module: &shader_b,
            entry_point: "main",
        });

        let radix_digit_places = 4; // Example, adjust as needed

        let radix_base = 256; // Example, adjust as needed
        let max_tile_count_c = 16; // Example, adjust as needed

        let shader_c = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/radix_sort_c.wgsl").into()),
        });

        let radix_sort_c_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Radix Sort C Pipeline"),
            layout: None,
            module: &shader_c,
            entry_point: "main",
        });

        let workgroup_entries_c = 64; // Example, adjust as needed

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("MyShader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders.wgsl"))),
            // defines: vec![
            //     ("RADIX_BASE", "16u"),
            //     ("RADIX_DIGIT_PLACES", "4u"),
            //     // ...
            // ]
            // .into_iter()
            // .collect(),
        });

        Ok(Self {
            sorting_bind_group_layout,
            compute_pipeline_layout,
            pipeline: todo!(),
            bind_group_layout: todo!(),
            vertex_buffer: todo!(),
            config,
            entry_buffer_a,
            uniform_buffer,
            sorting_buffer,
            sorting_buffer_size,
            radix_sort_a_pipeline,
            workgroup_entries_a,
            radix_sort_b_pipeline,
            radix_digit_places,
            radix_base,
            max_tile_count_c,
            radix_sort_c_pipeline,
            workgroup_entries_c,
        })
    }

    pub fn render(&self, encoder: &mut CommandEncoder, view: &TextureView, queue: &Queue, scene: &mut Scene) {
        let splat_count = scene.splat_count;
        let entries = scene.splat_data.iter().map(|splat| splat.into()).collect::<Vec<SplatEntry>>();

        // CPU depth sorting
        if matches!(self.config.depth_sorting, DepthSorting::Cpu) {
            scene.splat_data.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap());
        }

        let mut visible_entries = vec![];

        for (index, splat) in scene.splat_data.iter().enumerate() {
            let clip_space_position = scene.camera.get_clip_space_position(&Vec3::from(splat.center));
            if clip_space_position.z > 0.0 && clip_space_position.z < 1.0 {
                if clip_space_position.x.abs() < self.config.frustum_culling_tolerance
                    && clip_space_position.y.abs() < self.config.frustum_culling_tolerance
                {
                    visible_entries.push(index as u32);
                }
            }
        }

        queue.write_buffer(&self.entry_buffer_a, 0, bytemuck::cast_slice(&entries));

        let uniforms = Uniforms {
            projection: scene.camera.projection.to_cols_array_2d(),
            view: scene.camera.view.to_cols_array_2d(),
            params: [
                scene.camera.z_near,
                scene.camera.z_far,
                self.config.frustum_culling_tolerance,
                self.config.ellipse_margin,
            ],
            counts: [self.config.splat_scale, splat_count as f32, visible_entries.len() as f32, 0.0],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // GPU depth sorting
        if matches!(self.config.depth_sorting, DepthSorting::Gpu | DepthSorting::GpuIndirectDraw) {
            encoder.clear_buffer(&self.sorting_buffer, 0, None);

            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("radix_sort_a") });
                compute_pass.set_pipeline(&self.radix_sort_a_pipeline);
                compute_pass.dispatch_workgroups(
                    (((splat_count as u32) + self.workgroup_entries_a - 1) / self.workgroup_entries_a) as u32,
                    1,
                    1,
                );
            }

            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("radix_sort_b") });
                compute_pass.set_pipeline(&self.radix_sort_b_pipeline);
                compute_pass.dispatch_workgroups(1, self.radix_digit_places as u32, 1);
            }

            for pass_index in 0..self.radix_digit_places {
                encoder.copy_buffer_to_buffer(
                    &self.sorting_buffer,
                    (self.radix_base as u64 * self.max_tile_count_c as u64 * std::mem::size_of::<u32>() as u64) as u64,
                    &self.sorting_buffer,
                    0,
                    (self.radix_base as u64 * self.max_tile_count_c as u64 * std::mem::size_of::<u32>() as u64) as u64,
                );

                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("radix_sort_c") });
                compute_pass.set_pipeline(&self.radix_sort_c_pipeline);
                compute_pass.dispatch_workgroups(
                    1,
                    (((splat_count as u32) + self.workgroup_entries_c - 1) / self.workgroup_entries_c) as u32,
                    1,
                );
            }
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
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

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, scene.render_bind_group.as_ref().unwrap(), &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        if matches!(self.config.depth_sorting, DepthSorting::GpuIndirectDraw) {
            render_pass.draw_indirect(&self.sorting_buffer, (self.sorting_buffer_size - std::mem::size_of::<u32>() * 5) as u64);
        } else {
            render_pass.draw(0..6, 0..splat_count as u32);
        }
    }
}

#[derive(Pod, Zeroable, Copy, Clone)]
#[repr(C)]
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
