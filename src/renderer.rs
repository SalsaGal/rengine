use std::sync::OnceLock;

use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::{dpi::PhysicalSize, window::Window};

#[derive(Debug)]
pub struct RendererGlobals {
    pub(crate) device: Device,
    pub(crate) queue: Queue,
}

static GLOBALS: OnceLock<RendererGlobals> = OnceLock::new();

impl RendererGlobals {
    pub(crate) fn get() -> &'static Self {
        GLOBALS.get().expect("renderer must be created")
    }
}

pub struct Renderer {
    surface: Surface,
    config: SurfaceConfiguration,

    pub window: Window,
}

impl Renderer {
    pub(crate) async fn new(window: Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        });
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adaptor = instance
            .enumerate_adapters(wgpu::Backends::PRIMARY)
            .find(|a| a.is_surface_supported(&surface))
            .unwrap();
        let (device, queue) = adaptor
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();
        let surface_caps = surface.get_capabilities(&adaptor);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats[0],
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        GLOBALS.set(RendererGlobals { device, queue }).unwrap();

        Self {
            surface,
            config,
            window,
        }
    }

    pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface
            .configure(&RendererGlobals::get().device, &self.config);
    }

    pub(crate) fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let current = self.surface.get_current_texture()?;
        let view = current
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = RendererGlobals::get()
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("color pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        RendererGlobals::get()
            .queue
            .submit(std::iter::once(encoder.finish()));
        current.present();

        Ok(())
    }
}
