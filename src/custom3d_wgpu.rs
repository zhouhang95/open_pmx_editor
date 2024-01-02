use std::{path::Path, sync::Arc};
use glam::*;
use eframe::{
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu, RenderState},
};

use crate::{camera::{Camera, CameraUniform}, grid::{ GridRenderResources, CustomGridCallback}};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: Vec3,
    color: Vec3,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

const VERTICES: &[Vertex] = &[
    Vertex { position: vec3(-0.0868241, 0.49240386, 0.0), color: vec3(0.5, 0.0, 0.5) }, // A
    Vertex { position: vec3(-0.49513406, 0.06958647, 0.0), color: vec3(0.5, 0.0, 0.5) }, // B
    Vertex { position: vec3(-0.21918549, -0.44939706, 0.0), color: vec3(0.5, 0.0, 0.5) }, // C
    Vertex { position: vec3(0.35966998, -0.3473291, 0.0), color: vec3(0.5, 0.0, 0.5) }, // D
    Vertex { position: vec3(0.44147372, 0.2347359, 0.0), color: vec3(0.5, 0.0, 0.5) }, // E
];

const INDICES: &[u32] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

const VERTEX_BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
};

pub struct Custom3d {
    camera: Camera,
    wgpu_render_state: RenderState,

}

impl Custom3d {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Self {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap().clone();

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our `Custom3D` struct, we insert it into the
        // `paint_callback_resources` type map, which is stored alongside the render pass.
        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(TriangleRenderResources::new(wgpu_render_state.device.clone(), wgpu_render_state.target_format.into()));
        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(GridRenderResources::new(wgpu_render_state.device.clone(), wgpu_render_state.target_format.into()));
        Self {
            camera: Camera::new(),
            wgpu_render_state,
        }
    }
}

// Callbacks in egui_wgpu have 3 stages:
// * prepare (per callback impl)
// * finish_prepare (once)
// * paint (per callback impl)
//
// The prepare callback is called every frame before paint and is given access to the wgpu
// Device and Queue, which can be used, for instance, to update buffers and uniforms before
// rendering.
// If [`egui_wgpu::Renderer`] has [`egui_wgpu::FinishPrepareCallback`] registered,
// it will be called after all `prepare` callbacks have been called.
// You can use this to update any shared resources that need to be updated once per frame
// after all callbacks have been processed.
//
// On both prepare methods you can use the main `CommandEncoder` that is passed-in,
// return an arbitrary number of user-defined `CommandBuffer`s, or both.
// The main command buffer, as well as all user-defined ones, will be submitted together
// to the GPU in a single call.
//
// The paint callback is called after finish prepare and is given access to egui's main render pass,
// which can be used to issue draw commands.
struct CustomTriangleCallback {
    camera_uniform: CameraUniform,
}

impl egui_wgpu::CallbackTrait for CustomTriangleCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &TriangleRenderResources = resources.get().unwrap();
        resources.prepare(device, queue, self.camera_uniform);
        Vec::new()
    }

    fn paint<'a>(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        resources: &'a egui_wgpu::CallbackResources,
    ) {
        let resources: &TriangleRenderResources = resources.get().unwrap();
        resources.paint(render_pass);
    }
}


impl Custom3d {
    pub fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size_before_wrap(), egui::Sense::drag());

        // manipulate camera
        {
            if ui.input(|i| i.modifiers.shift) {
                self.camera.pan(response.drag_delta().x, response.drag_delta().y);
            } else {
                self.camera.orbit(response.drag_delta().x, response.drag_delta().y);
            }
            let scroll_delta = ui.input(|i| i.scroll_delta.y);
            self.camera.dolly(if scroll_delta > 0.0 { 1.0 } else if scroll_delta < 0.0 { -1.0} else { 0.0 });
            self.camera.aspect_ratio = rect.aspect_ratio();
        }
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            CustomTriangleCallback { camera_uniform: CameraUniform::from_camera(&self.camera) },
        ));
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            CustomGridCallback { camera_uniform: CameraUniform::from_camera(&self.camera) },
        ));
    }
}

struct TriangleRenderResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vert_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl TriangleRenderResources {
    pub fn new(device: Arc<wgpu::Device>, color_target_state: wgpu::ColorTargetState) -> Self {
        let shader_path = Path::new("shader/grid.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Grid Shader"),
            source: wgpu::ShaderSource::Wgsl(std::fs::read_to_string(shader_path).unwrap().into()),
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

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("grid_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VERTEX_BUFFER_LAYOUT],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(color_target_state)],
            }),
            primitive: wgpu::PrimitiveState {
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
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("custom3d"),
            contents: bytemuck::cast_slice(&[CameraUniform::new()]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let vert_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("custom3d vert"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("custom3d"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        Self {
            pipeline,
            bind_group,
            uniform_buffer,
            vert_buffer,
            index_buffer,
            num_indices: INDICES.len() as _,
        }
    }
    fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, camera_uniform: CameraUniform) {
        // Update our uniform buffer with the angle from the UI
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
    }

    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        // Draw our triangle!
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vert_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}
