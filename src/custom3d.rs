use std::{path::Path, sync::Arc, collections::HashMap};
use egui::mutex::Mutex;
use glam::*;
use eframe::{
    egui_wgpu::{self, wgpu::{self, util::DeviceExt}, RenderState, ScreenDescriptor},
    wgpu::{BlendState, ColorTargetState, ColorWrites, Face},
};

use image::{ImageReader, RgbaImage};
use std::sync::LazyLock;

#[allow(dead_code)]
pub const IMAGE_TOONS: LazyLock<Vec<RgbaImage>> = LazyLock::new(|| {
    let mut res = Vec::new();
    for i in 1..=10 {
        let reader = ImageReader::open(format!("assets/toons/toon{:02}.bmp", i)).unwrap();
        let img = reader.decode().unwrap().into_rgba8();
        res.push(img);
    }
    res
});

use crate::{camera::{Camera, CameraUniform}, grid::{ GridRenderResources, CustomGridCallback}, format::pmx::{Pmx, DrawFlags}, texture::TextureWrapper};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    pos: Vec3,
    nrm: Vec3,
    uv: Vec2,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

const VERTEX_BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2],
};

#[derive(Default, Clone, Copy)]
pub struct DrawFlag {
    pub planer: bool,
    pub wireframe: bool,
    pub gray: bool,
    pub use_texture: bool,
}

#[allow(dead_code)]
#[derive(Default, Clone, Copy)]
struct MatUniform {
    pub diffuse: Vec4,
    pub specular: Vec4,
    pub ambient: Vec4,
}
unsafe impl bytemuck::Pod for MatUniform {}
unsafe impl bytemuck::Zeroable for MatUniform {}

pub struct Custom3d {
    camera: Camera,
    wgpu_render_state: RenderState,
    pub draw_flag: DrawFlag,
    pub filters: Vec<(String, bool)>,
    pub show_material_filter: bool,
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
            .insert(GridRenderResources::new(wgpu_render_state.device.clone(), wgpu_render_state.target_format.into()));
        Self {
            camera: Camera::new(),
            wgpu_render_state,
            draw_flag: Default::default(),
            filters: Vec::new(),
            show_material_filter: false,
        }
    }
    pub fn load_mesh(&mut self, pmx: Arc<Mutex<Pmx>>) {
        let pmx = pmx.lock();
        for m in &pmx.mats {
            self.filters.push((m.name.clone(), true));
        }
        self.wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(TriangleRenderResources::new(
                self.wgpu_render_state.clone(),
                pmx.clone(),
            ));
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
    draw_wireframe: bool,
    filters: Vec<(String, bool)>,
}

impl egui_wgpu::CallbackTrait for CustomTriangleCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(resources) = resources.get_mut::<TriangleRenderResources>() {
            resources.prepare(device, queue, self.camera_uniform, self.draw_wireframe, self.filters.clone());
        }
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
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
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            self.camera.dolly(if scroll_delta > 0.0 { 1.0 } else if scroll_delta < 0.0 { -1.0} else { 0.0 });
            self.camera.aspect_ratio = rect.aspect_ratio();
        }
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            CustomTriangleCallback {
                camera_uniform: CameraUniform::from_camera(&self.camera, self.draw_flag),
                draw_wireframe: self.draw_flag.wireframe,
                filters: self.filters.clone(),
            },
        ));
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            CustomGridCallback { camera_uniform: CameraUniform::from_camera(&self.camera, self.draw_flag) },
        ));
    }
}

struct TriangleRenderResources {
    pipeline: wgpu::RenderPipeline,
    cull_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vert_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    mat_bind_groups: Vec<wgpu::BindGroup>,
    pmx: Pmx,
    wireframe_pipeline: wgpu::RenderPipeline,
    draw_wireframe: bool,
    filters: Vec<(String, bool)>,
}

impl TriangleRenderResources {
    pub fn new(
        render_state: RenderState,
        pmx: Pmx,
    ) -> Self {
        let device = render_state.device.clone();
        let color_target_state = render_state.target_format.clone();
        let queue = render_state.queue.clone();

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });
        
        let tex_images = pmx.load_tex();
        let mut texture_wrappers = HashMap::new();
        for (i, tex_image) in &tex_images {
            let texture = TextureWrapper::from_image(&device, &queue, tex_image, None);
            texture_wrappers.insert(*i, texture);
        }
        let mut toon_texture_wrappers = Vec::new();
        for tex_image in IMAGE_TOONS.iter() {
            toon_texture_wrappers.push(TextureWrapper::from_image(&device, &queue, tex_image, None));
        }
        let mut mat_bind_groups = Vec::new();
        for mat in &pmx.mats {
            let mat_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("mat uniform"),
                contents: bytemuck::cast_slice(&[MatUniform {
                    diffuse: mat.diffuse,
                    specular: mat.specular,
                    ambient: mat.ambient.extend(0.0),
                }]),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });
            let tex_index = if tex_images.contains_key(&mat.tex_index) { mat.tex_index } else { -1 };
            let texture = &texture_wrappers[&tex_index];
            let toon_texture = match mat.toon {
                crate::format::pmx::Toon::Tex(i) => {
                    let tex_index = if tex_images.contains_key(&i) { i } else { -1 };
                    &texture_wrappers[&tex_index]
                },
                crate::format::pmx::Toon::Inner(i) => {
                    &toon_texture_wrappers[i as usize]
                },
            };
            let mat_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: mat_uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&toon_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&toon_texture.sampler),
                    },
                ],
                label: Some("mat_bind_group"),
            });
            mat_bind_groups.push(mat_bind_group);
        }

        let shader_path = Path::new("shader/mesh.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Mesh Shader"),
            source: wgpu::ShaderSource::Wgsl(std::fs::read_to_string(shader_path).unwrap().into()),
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("custom3d"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
            bind_group_layouts: &[
                &bind_group_layout,
                &texture_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let wireframe_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mesh_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VERTEX_BUFFER_LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "wireframe_main",
                targets: &[Some(color_target_state.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                polygon_mode: wgpu::PolygonMode::Line,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: -400,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mesh_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VERTEX_BUFFER_LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: color_target_state,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
            cache: None,
        });
        let cull_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mesh_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VERTEX_BUFFER_LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: color_target_state,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(Face::Back),
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
            cache: None,
        });
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("custom3d"),
            contents: bytemuck::cast_slice(&[CameraUniform::default()]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let mut verts = Vec::new();
        for v in &pmx.verts {
            verts.push(Vertex { pos: v.pos, nrm: v.nrm , uv: v.uv});
        }
        let mut idxs = Vec::new();
        for i in &pmx.faces {
            idxs.push(i[0]);
            idxs.push(i[1]);
            idxs.push(i[2]);
        }

        let vert_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("custom3d vert"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&idxs),
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
            cull_pipeline,
            bind_group,
            uniform_buffer,
            vert_buffer,
            index_buffer,
            mat_bind_groups,
            pmx,
            wireframe_pipeline,
            draw_wireframe: false,
            filters: Vec::new(),
        }
    }
    fn prepare(
        &mut self, _device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera_uniform: CameraUniform,
        draw_wireframe: bool,
        filters: Vec<(String, bool)>,
    ) {
        self.draw_wireframe = draw_wireframe;
        self.filters = filters;
        // Update our uniform buffer with the angle from the UI
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        // Draw our triangle!
        render_pass.set_vertex_buffer(0, self.vert_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        for (i, mat) in self.pmx.mats.iter().enumerate() {
            render_pass.set_bind_group(1, &self.mat_bind_groups[i], &[]);
            if self.filters[i].1 == false {
                continue;
            }
            if mat.diffuse.w == 0.0 {
                continue;
            }
            let mut start_index = 0;
            for j in 0..i {
                start_index += self.pmx.mats[j].associated_face_count;
            }
            let face_count = mat.associated_face_count;
            let indices = (start_index * 3)..(start_index * 3 + face_count * 3);
            if mat.draw_flag.contains(DrawFlags::NO_CULL) {
                render_pass.set_pipeline(&self.pipeline);
            } else {
                render_pass.set_pipeline(&self.cull_pipeline);
            }
            render_pass.draw_indexed(indices.clone(), 0, 0..1);
            if self.draw_wireframe {
                render_pass.set_pipeline(&self.wireframe_pipeline);
                render_pass.draw_indexed(indices, 0, 0..1);
            }
        }
    }
}
