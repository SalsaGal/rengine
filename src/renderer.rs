use std::sync::OnceLock;

use dirtytype::Dirty;
use glam::{vec2, Mat4};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Device, Queue, Surface, SurfaceConfiguration,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::sprite::{Sprite, SpriteType};

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

    pub projection: Dirty<Projection>,
    projection_buffer: wgpu::Buffer,
    projection_bind_group: wgpu::BindGroup,
    pub sprites: Vec<Sprite>,
    color_pipeline: wgpu::RenderPipeline,
    texture_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub(crate) async fn new(window: Window, projection: Projection) -> Self {
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

        let projection_buffer =
            RendererGlobals::get()
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[Self::projection_to_mat4(
                        projection,
                        window.inner_size(),
                    )]),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                });
        let projection_bind_group_layout = RendererGlobals::get().device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
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
            },
        );
        let projection_bind_group =
            RendererGlobals::get()
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("projection"),
                    layout: &projection_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            projection_buffer.as_entire_buffer_binding(),
                        ),
                    }],
                });

        Self {
            surface,
            config,

            window,

            projection: Dirty::new(projection),
            projection_buffer,
            projection_bind_group,
            color_pipeline: Sprite::color_pipeline(&projection_bind_group_layout),
            texture_pipeline: Sprite::texture_pipeline(&projection_bind_group_layout),
            sprites: vec![],
        }
    }

    pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        RendererGlobals::get().queue.write_buffer(
            &self.projection_buffer,
            0,
            bytemuck::cast_slice(&[Self::projection_to_mat4(
                *self.projection,
                self.window.inner_size(),
            )]),
        );
        self.surface
            .configure(&RendererGlobals::get().device, &self.config);
    }

    pub(crate) fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if self.projection.dirty {
            // Can't use `Dirty::clean` because it requires weird mutability issues
            RendererGlobals::get().queue.write_buffer(
                &self.projection_buffer,
                0,
                bytemuck::cast_slice(&[Self::projection_to_mat4(
                    *self.projection,
                    self.window.inner_size(),
                )]),
            );
            self.projection.dirty = false;
        }

        let current = self.surface.get_current_texture()?;
        let view = current
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = RendererGlobals::get()
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            render_pass.set_bind_group(0, &self.projection_bind_group, &[]);
            for model in &self.sprites {
                match &model.ty {
                    SpriteType::Color => render_pass.set_pipeline(&self.color_pipeline),
                    SpriteType::Texture(texture) => {
                        render_pass.set_pipeline(&self.texture_pipeline);
                        render_pass.set_bind_group(1, texture, &[]);
                    }
                }
                render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(model.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..model.index_count, 0, 0..1);
            }
        }

        RendererGlobals::get()
            .queue
            .submit(std::iter::once(encoder.finish()));
        current.present();

        Ok(())
    }

    fn projection_to_mat4(projection: Projection, size: PhysicalSize<u32>) -> Mat4 {
        let size = vec2(size.width as f32, size.height as f32);
        match projection {
            Projection::Absolute(width, height) => Mat4::orthographic_rh(
                -width / 2.0,
                width / 2.0,
                -height / 2.0,
                height / 2.0,
                -10.0,
                10.0,
            ),
            Projection::FixedWidth(width) => Mat4::orthographic_rh(
                -width / 2.0,
                width / 2.0,
                -width * (size.y / size.x) / 2.0,
                width * (size.y / size.x) / 2.0,
                -10.0,
                10.0,
            ),
            Projection::FixedHeight(height) => Mat4::orthographic_rh(
                -height * (size.x / size.y) / 2.0,
                height * (size.x / size.y) / 2.0,
                -height / 2.0,
                height / 2.0,
                -10.0,
                10.0,
            ),
            _ => todo!(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Projection {
    Absolute(f32, f32),
    FixedWidth(f32),
    FixedHeight(f32),
    FixedMinimum(f32, f32),
}

impl Default for Projection {
    fn default() -> Self {
        Self::Absolute(2.0, 2.0)
    }
}
