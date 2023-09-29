use std::sync::OnceLock;

use dirtytype::Dirty;
use glam::{vec2, Mat4};
use slab::Slab;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Device, Queue, Surface, SurfaceConfiguration,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    camera::Camera,
    sprite::{Sprite, SpriteType},
};

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

    pub background: wgpu::Color,
    pub camera: Dirty<Camera>,
    camera_buffer: wgpu::Buffer,
    pub projection: Dirty<Projection>,
    projection_buffer: wgpu::Buffer,
    projection_bind_group: wgpu::BindGroup,
    pub sprites: Slab<Sprite>,
    color_pipeline: wgpu::RenderPipeline,
    texture_pipeline: wgpu::RenderPipeline,
    depth_view: wgpu::TextureView,
}

impl Renderer {
    pub(crate) async fn new(window: Window, camera: Camera, projection: Projection) -> Self {
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
        let camera_buffer =
            RendererGlobals::get()
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[Mat4::from(&camera)]),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                });
        let projection_bind_group_layout = RendererGlobals::get().device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
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
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            },
        );
        let projection_bind_group =
            RendererGlobals::get()
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("projection"),
                    layout: &projection_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(
                                projection_buffer.as_entire_buffer_binding(),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer(
                                camera_buffer.as_entire_buffer_binding(),
                            ),
                        },
                    ],
                });

        Self {
            surface,
            config,

            background: wgpu::Color::BLACK,
            camera: Dirty::new(camera),
            camera_buffer,
            projection: Dirty::new(projection),
            projection_buffer,
            projection_bind_group,
            color_pipeline: Sprite::color_pipeline(&projection_bind_group_layout),
            texture_pipeline: Sprite::texture_pipeline(&projection_bind_group_layout),
            depth_view: Self::make_depth_texture(window.inner_size()),
            sprites: Slab::default(),

            window,
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
        self.depth_view = Self::make_depth_texture(size);
        self.surface
            .configure(&RendererGlobals::get().device, &self.config);
    }

    pub(crate) fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Can't use `Dirty::clean` because it requires weird mutability issues
        if self.camera.dirty {
            RendererGlobals::get().queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[Mat4::from(&*self.camera)]),
            );
            self.projection.dirty = false;
        }
        if self.projection.dirty {
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
                        load: wgpu::LoadOp::Clear(self.background),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_bind_group(0, &self.projection_bind_group, &[]);
            for (_, model) in &mut self.sprites {
                model.transforms.clean(|t| {
                    RendererGlobals::get().queue.write_buffer(
                        &model.transform_buffer,
                        0,
                        bytemuck::cast_slice(&t.iter().map(Mat4::from).collect::<Vec<_>>()),
                    )
                });

                match &model.ty {
                    SpriteType::Color => render_pass.set_pipeline(&self.color_pipeline),
                    SpriteType::Texture(texture) => {
                        render_pass.set_pipeline(&self.texture_pipeline);
                        render_pass.set_bind_group(1, texture, &[]);
                    }
                }
                render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, model.transform_buffer.slice(..));
                render_pass
                    .set_index_buffer(model.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..model.index_count, 0, 0..model.transform_count);
            }
        }

        RendererGlobals::get()
            .queue
            .submit(std::iter::once(encoder.finish()));
        current.present();

        Ok(())
    }

    fn projection_to_mat4(projection: Projection, size: PhysicalSize<u32>) -> Mat4 {
        let window_size = vec2(size.width as f32, size.height as f32);
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
                -width * (window_size.y / window_size.x) / 2.0,
                width * (window_size.y / window_size.x) / 2.0,
                -10.0,
                10.0,
            ),
            Projection::FixedHeight(height) => Mat4::orthographic_rh(
                -height * (window_size.x / window_size.y) / 2.0,
                height * (window_size.x / window_size.y) / 2.0,
                -height / 2.0,
                height / 2.0,
                -10.0,
                10.0,
            ),
            Projection::FixedMinimum(width, height) => {
                if window_size.x > window_size.y {
                    Self::projection_to_mat4(Projection::FixedHeight(height), size)
                } else {
                    Self::projection_to_mat4(Projection::FixedWidth(width), size)
                }
            }
        }
    }

    fn make_depth_texture(size: PhysicalSize<u32>) -> wgpu::TextureView {
        RendererGlobals::get()
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("depth"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
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
