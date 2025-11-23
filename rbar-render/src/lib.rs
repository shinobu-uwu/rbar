mod context;
mod structures;

use std::ptr::NonNull;

use anyhow::Result;
use bytemuck::{bytes_of, cast_slice};
use wayland_client::backend::ObjectId;
use wgpu::{
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, Buffer, BufferDescriptor, BufferUsages,
    ColorTargetState, CompositeAlphaMode, DeviceDescriptor, FragmentState, IndexFormat, Instance,
    InstanceDescriptor, LoadOp, Operations, PipelineCompilationOptions, PipelineLayoutDescriptor,
    PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderStages, StoreOp, Surface,
    SurfaceConfiguration,
    SurfaceTargetUnsafe::RawHandle,
    TextureFormat, TextureUsages, VertexState, include_wgsl,
    rwh::{RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle},
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    context::WgpuContext,
    structures::{Color, Globals, Position, Size, WidgetInstance},
};

const QUAD_VERTICES: &[Position] = &[
    Position(-0.5, -0.5), // bottom-left
    Position(0.5, -0.5),  // bottom-right
    Position(-0.5, 0.5),  // top-left
    Position(0.5, 0.5),   // top-right
];

const QUAD_INDICES: &[u16] = &[0, 1, 2, 2, 1, 3];

pub struct Renderer {
    instance: Instance,
    context: WgpuContext,
}

pub struct SurfaceRenderer {
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
    context: WgpuContext,
    pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    global_buffer: wgpu::Buffer,
    widget_buffer: wgpu::Buffer,
    widget_count: u32,
    bind_group: BindGroup,
    num_indices: u32,
}

impl Renderer {
    pub async fn new() -> Result<Self> {
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await?;
        let (device, queue) = adapter.request_device(&DeviceDescriptor::default()).await?;
        let context = WgpuContext::new(device, queue, adapter);

        Ok(Self { instance, context })
    }

    fn create_pipeline(
        &self,
        format: TextureFormat,
        bind_group_layout: &BindGroupLayout,
    ) -> RenderPipeline {
        let shader = self
            .context
            .device
            .create_shader_module(include_wgsl!("../shaders/quad.wgsl"));

        let layout = self
            .context
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("pipeline layout"),
                bind_group_layouts: &[bind_group_layout],
                push_constant_ranges: &[],
            });

        self.context
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("pipeline"),
                layout: Some(&layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Position::descriptor(), WidgetInstance::descriptor()],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
    }

    pub fn create_surface_renderer<'a>(
        &'a self,
        backend: &wayland_client::backend::Backend,
        surface_id: ObjectId,
        width: u32,
        height: u32,
    ) -> SurfaceRenderer {
        let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(backend.display_ptr() as *mut _).unwrap(),
        ));
        let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(surface_id.as_ptr() as *mut _).unwrap(),
        ));

        let surface = unsafe {
            self.instance
                .create_surface_unsafe(RawHandle {
                    raw_display_handle,
                    raw_window_handle,
                })
                .unwrap()
        };
        let surface_caps = surface.get_capabilities(&self.context.adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let alpha_mode = surface_caps
            .alpha_modes
            .iter()
            .find(|&m| *m == CompositeAlphaMode::PreMultiplied)
            .copied()
            .unwrap_or(CompositeAlphaMode::Auto);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            alpha_mode,
            present_mode: surface_caps.present_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&self.context.device, &config);
        let bind_group_layout =
            self.context
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Bind Group"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        let global_buffer = self.context.device.create_buffer(&BufferDescriptor {
            label: Some("Global buffer"),
            size: size_of::<Globals>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = self.context.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Globals bind group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: global_buffer.as_entire_binding(),
            }],
        });
        let pipeline = self.create_pipeline(surface_format, &bind_group_layout);
        let vertex_buffer = self
            .context
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(QUAD_VERTICES),
                usage: BufferUsages::VERTEX,
            });
        let index_buffer = self
            .context
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: cast_slice(QUAD_INDICES),
                usage: BufferUsages::INDEX,
            });
        let widget_buffer = self
            .context
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Widget Instance Buffer"),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                contents: cast_slice(&[WidgetInstance::new(
                    0.0,
                    0.0,
                    100.0,
                    30.0,
                    Color(1.0, 0.0, 0.0, 1.0),
                    0.0,
                )]),
            });

        SurfaceRenderer {
            surface,
            config,
            pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: QUAD_INDICES.len() as u32,
            context: self.context.clone(),
            global_buffer,
            widget_buffer,
            widget_count: 1,
            bind_group,
        }
    }
}

impl SurfaceRenderer {
    pub fn render(&mut self) -> Result<()> {
        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());

        let mut encoder = self
            .context
            .device
            .create_command_encoder(&Default::default());

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, self.widget_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw_indexed(0..self.num_indices, 0, 0..self.widget_count);
        }

        self.context.queue.submit([encoder.finish()]);
        frame.present();

        Ok(())
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.context.device, &self.config);
        let new_globals = Globals {
            resolution: Size(width as f32, height as f32),
        };
        self.context
            .queue
            .write_buffer(&self.global_buffer, 0, bytes_of(&new_globals));
    }
}
