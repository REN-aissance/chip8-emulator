use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use wgpu::{
    util::DeviceExt, Color, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor,
    Features, FragmentState, FrontFace, IndexFormat, Instance, Limits, LoadOp, MultisampleState,
    Operations, PipelineLayoutDescriptor, PolygonMode, PowerPreference, PresentMode,
    PrimitiveState, PrimitiveTopology, Queue, RenderPass, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
    ShaderModuleDescriptor, ShaderSource, StoreOp, Surface, SurfaceConfiguration, TextureView,
    TextureViewDescriptor, VertexState,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::ASPECT_RATIO;

pub struct Renderer<'a> {
    size: PhysicalSize<u32>,
    instance: Instance,
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
    vertex_buffer: [Vertex; 4],
}

impl<'a> Renderer<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = Instance::default();
        let surface = instance
            .create_surface(window)
            .expect("Could not create surface");

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Could not find suitable adapter");

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Unable to request device from adapter");

        let mut config = surface
            .get_default_config(&adapter, size.width, size.height)
            .expect("No default config for surface");
        config.present_mode = PresentMode::AutoVsync;

        surface.configure(&device, &config);

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("screen_shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/shader.wgsl"))),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("my_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("my_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vertex",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fragment",
                targets: &[Some(config.format.into())],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(IndexFormat::Uint32),
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            multisample: MultisampleState::default(),
            depth_stencil: None,
            multiview: None,
        });

        let mut out = Self {
            size,
            instance,
            surface,
            device,
            queue,
            config,
            pipeline,
            vertex_buffer: [Vertex::default(); 4],
        };
        out.reset_vertex_buffer();
        out
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width < 1 || new_size.height < 1 {
            return;
        }
        self.instance.poll_all(true);
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.reset_vertex_buffer();
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self) {
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.vertex_buffer),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let frame = self
            .surface
            .get_current_texture()
            .expect("Could not get surface texture");
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        {
            let mut render_pass = Self::get_render_pass(&mut encoder, &view);
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..4, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn get_render_pass<'b>(
        encoder: &'b mut CommandEncoder,
        view: &'b TextureView,
    ) -> RenderPass<'b> {
        let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass
    }

    pub fn reset_vertex_buffer(&mut self) {
        let PhysicalSize { width, height } = self.size;
        let (width, height) = (width as f32, height as f32);
        let window_aspect_ratio = width / height;

        let (x1, x2, y1, y2) = if ASPECT_RATIO > window_aspect_ratio {
            //Vertical letterboxing
            let height = (width / ASPECT_RATIO) / height;
            (-1.0, 1.0, -height, height)
        } else {
            //Horizontal letterboxing
            let width = (height * ASPECT_RATIO) / width;
            (-width, width, -1.0, 1.0)
        };

        self.vertex_buffer = [[x1, y1], [x1, y2], [x2, y1], [x2, y2]].map(|p| Vertex {
            position: p,
            color: [p[0], p[1], 0.0],
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3];
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}
