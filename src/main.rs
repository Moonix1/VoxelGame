use std::{process::exit, sync::Arc};

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window
};

use log::{error, warn};

struct App<'a> {
    surface: Option<wgpu::Surface<'a>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    size: Option<PhysicalSize<u32>>,

    window: Option<Arc<Window>>,
}

impl<'a> App<'a> {
    fn init() -> Self {
        Self {
            surface:    None,
            device:     None,
            queue:      None,
            config:     None,
            size:       None,

            window:     None,
        }
    }
    
    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = Some(new_size);
            
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

    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.as_ref().unwrap().get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.as_ref().unwrap()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        self.queue.as_ref().unwrap().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
        
        self.size = Some(window.inner_size());
        
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        });
        
        let surface = instance.create_surface(window.clone()).unwrap();

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
            width: self.size.unwrap().width,
            height: self.size.unwrap().height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.window = Some(window);
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
        ) {
        match event {
            _ if window_id == self.window.as_ref().unwrap().id() => if !self.input(&event) {
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
                        self.window.as_ref().unwrap().request_redraw();
        
                        self.update();
                        match self.render() {
                            Ok(_) => (),
        
                            Err(
                                wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                            ) => self.resize(self.size.unwrap()),
        
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