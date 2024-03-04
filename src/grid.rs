use std::{path::Path, sync::Arc};

use glam::*;
use eframe::{
    egui_wgpu::{self, wgpu::{self, util::DeviceExt}, ScreenDescriptor},
    wgpu::{ColorTargetState, Device},
};

use crate::camera::CameraUniform;
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: Vec3,
    color: Vec3,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

pub const GRID: &[Vertex] = &[
    // x axis
    Vertex { position: vec3(-5.0, 0.0, -5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 5.0, 0.0, -5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-5.0, 0.0, -4.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 5.0, 0.0, -4.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-5.0, 0.0, -3.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 5.0, 0.0, -3.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-5.0, 0.0, -2.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 5.0, 0.0, -2.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-5.0, 0.0, -1.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 5.0, 0.0, -1.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-5.0, 0.0,  0.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 0.0, 0.0,  0.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 0.0, 0.0,  0.0), color: vec3(0.8, 0.2, 0.2) },
    Vertex { position: vec3( 5.0, 0.0,  0.0), color: vec3(0.8, 0.2, 0.2) },
    Vertex { position: vec3(-5.0, 0.0,  1.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 5.0, 0.0,  1.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-5.0, 0.0,  2.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 5.0, 0.0,  2.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-5.0, 0.0,  3.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 5.0, 0.0,  3.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-5.0, 0.0,  4.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 5.0, 0.0,  4.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-5.0, 0.0,  5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 5.0, 0.0,  5.0), color: vec3(0.3, 0.3, 0.3) },
    // z axis
    Vertex { position: vec3(-5.0, 0.0, -5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-5.0, 0.0,  5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-4.0, 0.0, -5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-4.0, 0.0,  5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-3.0, 0.0, -5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-3.0, 0.0,  5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-2.0, 0.0, -5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-2.0, 0.0,  5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-1.0, 0.0, -5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3(-1.0, 0.0,  5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 0.0, 0.0, -5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 0.0, 0.0,  0.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 0.0, 0.0,  0.0), color: vec3(0.2, 0.2, 0.8) },
    Vertex { position: vec3( 0.0, 0.0,  5.0), color: vec3(0.2, 0.2, 0.8) },
    Vertex { position: vec3( 1.0, 0.0, -5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 1.0, 0.0,  5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 2.0, 0.0, -5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 2.0, 0.0,  5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 3.0, 0.0, -5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 3.0, 0.0,  5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 4.0, 0.0, -5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 4.0, 0.0,  5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 5.0, 0.0, -5.0), color: vec3(0.3, 0.3, 0.3) },
    Vertex { position: vec3( 5.0, 0.0,  5.0), color: vec3(0.3, 0.3, 0.3) },
];

pub const GRID_BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
};

pub struct GridRenderResources {
    pub grid_pipeline: wgpu::RenderPipeline,
    pub grid_bind_group: wgpu::BindGroup,
    pub grid_uniform_buffer: wgpu::Buffer,
    pub grid_vert_buffer: wgpu::Buffer,
}

impl GridRenderResources {
    pub fn new(device: Arc<Device>, color_target_state: ColorTargetState) -> Self {
        let grid_shader_path = Path::new("shader/grid.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Grid Shader"),
            source: wgpu::ShaderSource::Wgsl(std::fs::read_to_string(grid_shader_path).unwrap().into()),
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("custom3d"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("custom3d"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let grid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("grid_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[GRID_BUFFER_LAYOUT],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(color_target_state)],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        let grid_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("custom3d"),
            contents: bytemuck::cast_slice(&[CameraUniform::default()]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let grid_vert_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("custom3d vert"),
            contents: bytemuck::cast_slice(GRID),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let grid_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("custom3d"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: grid_uniform_buffer.as_entire_binding(),
            }],
        });
        Self {
            grid_pipeline,
            grid_bind_group,
            grid_uniform_buffer,
            grid_vert_buffer,
        }
    }

    fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, camera_uniform: CameraUniform) {
        queue.write_buffer(
            &self.grid_uniform_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
    }

    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        render_pass.set_pipeline(&self.grid_pipeline);
        render_pass.set_vertex_buffer(0, self.grid_vert_buffer.slice(..));
        render_pass.set_bind_group(0, &self.grid_bind_group, &[]);
        render_pass.draw(0..GRID.len() as _,  0..1);
    }
}

pub struct CustomGridCallback {
    pub camera_uniform: CameraUniform,
}

impl egui_wgpu::CallbackTrait for CustomGridCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &GridRenderResources = resources.get().unwrap();
        resources.prepare(device, queue, self.camera_uniform);
        Vec::new()
    }

    fn paint<'a>(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        resources: &'a egui_wgpu::CallbackResources,
    ) {
        let resources: &GridRenderResources = resources.get().unwrap();
        resources.paint(render_pass);
    }
}
