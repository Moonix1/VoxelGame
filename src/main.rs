use std::{process::exit};

use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
};

use log::{error, warn};

mod window;
mod camera;
mod texture;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32 ; 3],
    tex_coords: [f32 ; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute ; 2]
        = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const VERTICES: &[Vertex] = &[
    // Front
    Vertex { position: [0.0, 0.5, 0.0],    tex_coords: [1.0, 0.0] },
    Vertex { position: [-0.5, 0.5, 0.0],   tex_coords: [0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0],  tex_coords: [0.0, 1.0] },
    Vertex { position: [0.0, -0.5, 0.0],   tex_coords: [1.0, 1.0] },

    // Right
    Vertex { position: [0.0, 0.5, -0.5],    tex_coords: [1.0, 0.0] },
    Vertex { position: [0.0, 0.5, 0.0],   tex_coords: [0.0, 0.0] },
    Vertex { position: [0.0, -0.5, 0.0],  tex_coords: [0.0, 1.0] },
    Vertex { position: [0.0, -0.5, -0.5],   tex_coords: [1.0, 1.0] },

    // Back
    Vertex { position: [0.0, -0.5, -0.5],    tex_coords: [1.0, 1.0] },
    Vertex { position: [-0.5, -0.5, -0.5],   tex_coords: [0.0, 1.0] },
    Vertex { position: [-0.5, 0.5, -0.5],  tex_coords: [0.0, 0.0] },
    Vertex { position: [0.0, 0.5, -0.5],   tex_coords: [1.0, 0.0] },

    // Left
    Vertex { position: [-0.5, 0.5, 0.0],    tex_coords: [1.0, 0.0] },
    Vertex { position: [-0.5, 0.5, -0.5],   tex_coords: [0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, -0.5],  tex_coords: [0.0, 1.0] },
    Vertex { position: [-0.5, -0.5, 0.0],   tex_coords: [1.0, 1.0] },

    // Top
    Vertex { position: [0.0, 0.5, 0.0],    tex_coords: [1.0, 0.0] },
    Vertex { position: [0.0, 0.5, -0.5],   tex_coords: [0.0, 0.0] },
    Vertex { position: [-0.5, 0.5, -0.5],  tex_coords: [0.0, 1.0] },
    Vertex { position: [-0.5, 0.5, 0.0],   tex_coords: [1.0, 1.0] },

    // Bottom
    Vertex { position: [0.0, -0.5, 0.0],    tex_coords: [1.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0],   tex_coords: [0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, -0.5],  tex_coords: [0.0, 1.0] },
    Vertex { position: [0.0, -0.5, -0.5],   tex_coords: [1.0, 1.0] },
];

const INDICES: &[u16] = &[
    0, 1, 2,
    2, 3, 0,
    4, 5, 6,
    6, 7, 4,
    8, 9, 10,
    10, 11, 8,
    12, 13, 14,
    14, 15, 12,
    16, 17, 18,
    18, 19, 16,
    20, 21, 22,
    22, 23, 20,
];

#[allow(unused)]
struct App<'a> {
    surface: Option<wgpu::Surface<'a>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,

    render_pipeline: Option<wgpu::RenderPipeline>,

    camera: Option<camera::Camera>,
    camera_uniform: Option<camera::CameraUniform>,
    camera_buffer: Option<wgpu::Buffer>,
    camera_bind_group: Option<wgpu::BindGroup>,

    camera_controller: Option<camera::CameraController>,

    vertex_buffer: Option<wgpu::Buffer>,
    num_vertices: Option<u32>,

    index_buffer: Option<wgpu::Buffer>,
    num_indices: Option<u32>,

    diffuse_bind_group: Option<wgpu::BindGroup>,
    diffuse_texture: Option<texture::Texture>,

    window: Option<window::Window<'a>>,
}

impl<'a> App<'a> {
    fn init() -> Self {
        Self {
            surface:            None,
            device:             None,
            queue:              None,
            config:             None,

            render_pipeline:    None,
            
            camera:             None,
            camera_uniform:     None,
            camera_buffer:      None,
            camera_bind_group:  None,

            camera_controller:  None,

            vertex_buffer:      None,
            num_vertices:       None,

            index_buffer:       None,
            num_indices:        None,

            diffuse_bind_group: None,
            diffuse_texture:    None,

            window:             None,
        }
    }
    
    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.as_mut().unwrap().process_events(event)
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window.as_mut().unwrap().size = new_size;
            
            if let Some(config) = &mut self.config {
                config.width = new_size.width;
                config.height = new_size.height;
            }
            
            if let Some(surface) = &mut self.surface {
                surface.configure(
                    &self.device.as_ref().unwrap(),
                    &self.config.as_ref().unwrap()
                );
            }
        }
    }

    fn update(&mut self) {
        self.camera_controller.as_ref().unwrap().update_camera(self.camera.as_mut().unwrap());
        self.camera_uniform.as_mut().unwrap().update_view_proj(self.camera.as_ref().unwrap());
        self.queue.as_ref().unwrap().write_buffer(
            self.camera_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&[self.camera_uniform.unwrap()]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.as_ref().unwrap().get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.as_ref().unwrap()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                },
                            ),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline.as_ref().unwrap());
            render_pass.set_bind_group(0, self.diffuse_bind_group.as_ref().unwrap(), &[]);
            render_pass.set_bind_group(1, self.camera_bind_group.as_ref().unwrap(), &[]);
            render_pass.set_vertex_buffer(0, 
                self.vertex_buffer.as_ref().unwrap().slice(..)
            );
            render_pass.set_index_buffer(
                self.index_buffer.as_ref().unwrap().slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.draw_indexed(0..self.num_indices.unwrap(), 0, 0..1);
        }

        self.queue.as_ref().unwrap().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = window::Window::build(
            "Voxel Game",
            PhysicalSize { width: 720, height: 600 },
            event_loop,
        );
        
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        });
        
        let surface = instance.create_surface(window.core_window.clone()).unwrap();

        let adapter = pollster::block_on(
            async {
                instance.request_adapter(
                    &wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::default(),
                        compatible_surface: Some(&surface),
                        force_fallback_adapter: false,
                    },
                ).await.unwrap()
            }
        );

        let (device, queue) = pollster::block_on(
            async {
                adapter.request_device(
                &wgpu::DeviceDescriptor {
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                        label: None,
                        memory_hints: Default::default(),
                    },
                    None
                ).await.unwrap()
            }
        );

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.size.width,
            height: window.size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let diffuse_bytes = include_bytes!("../assets/happy-tree.png");
        let diffuse_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            diffuse_bytes,
            "happy-tree.png"
        ).unwrap();

        let texture_bind_group_layout
            = device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
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
                    ],
                    label: Some("texture_bind_group_layout"),
                }
            );

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        let camera = camera::Camera {
            eye: (0.0, 1.0, 1.3).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fov: 70.0,
            near: 0.1,
            far: 1000.0,
        };

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
                label: Some("camera_bind_group_layout"),
            }
        );

        let camera_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }
                ],
                label: Some("camera_bind_group"),
            }
        );

        let camera_controller = camera::CameraController::new(0.2);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let num_vertices = VERTICES.len() as u32;

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let num_indices = INDICES.len() as u32;

        self.surface            = Some(surface);
        self.device             = Some(device);
        self.queue              = Some(queue);
        self.config             = Some(config);
        self.render_pipeline    = Some(render_pipeline);
        self.camera             = Some(camera);
        self.camera_uniform     = Some(camera_uniform);
        self.camera_buffer      = Some(camera_buffer);
        self.camera_bind_group  = Some(camera_bind_group);
        self.camera_controller  = Some(camera_controller);
        self.vertex_buffer      = Some(vertex_buffer);
        self.num_vertices       = Some(num_vertices);
        self.index_buffer       = Some(index_buffer);
        self.num_indices        = Some(num_indices);
        self.diffuse_bind_group = Some(diffuse_bind_group);
        self.diffuse_texture    = Some(diffuse_texture);
        self.window             = Some(window);
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
        ) {
        match event {
            _ if window_id == self.window.as_ref().unwrap().core_window.id() => if !self.input(&event) {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                        event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                        ..
                    } => {
                        event_loop.exit();
                    },
                    
                    WindowEvent::Resized(new_size) => {
                        self.resize(new_size);
                    },
        
                    WindowEvent::RedrawRequested => {
                        self.window.as_ref().unwrap().core_window.request_redraw();
        
                        self.update();
                        match self.render() {
                            Ok(_) => (),
        
                            Err(
                                wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                            ) => self.resize(self.window.as_ref().unwrap().size),
        
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                error!("Out of memory!");
                                event_loop.exit();
                            },
        
                            Err(wgpu::SurfaceError::Timeout) => {
                                warn!("Surface Timout!");
                            }
                        };
                    },

                    _ => ()
                }
            },
            
            
            _ => (),
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    let mut app = App::init();
    match event_loop.run_app(&mut app) {
        Ok(_) => (),
        Err(_) => {
            error!("could not run app!");
            exit(1);
        }
    }
}