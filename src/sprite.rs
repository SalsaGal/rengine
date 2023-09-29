use std::mem::size_of;

use dirtytype::Dirty;
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

pub use glam::*;
pub use wgpu::Color;

use crate::{renderer::RendererGlobals, transform::Transform};

pub enum SpriteType {
    Color,
    Texture(wgpu::BindGroup),
}

pub trait Vertex: bytemuck::Pod + bytemuck::Zeroable {}

pub struct Sprite {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) index_count: u32,
    pub(crate) ty: SpriteType,
    pub transforms: Dirty<Vec<Transform>>,
    pub(crate) transform_buffer: wgpu::Buffer,
    pub(crate) transform_count: u32,
}

impl Sprite {
    #[must_use]
    pub fn new_polygon(
        vertices: &[impl Vertex],
        indices: &[u16],
        texture: Option<(&wgpu::TextureView, &wgpu::Sampler)>,
        transforms: Vec<Transform>,
    ) -> Self {
        Self {
            vertex_buffer: RendererGlobals::get().device.create_buffer_init(
                &BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ),
            index_buffer: RendererGlobals::get()
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(indices),
                    usage: wgpu::BufferUsages::INDEX,
                }),
            index_count: indices.len() as u32,
            ty: match texture {
                Some((view, sampler)) => {
                    SpriteType::Texture(RendererGlobals::get().device.create_bind_group(
                        &wgpu::BindGroupDescriptor {
                            label: None,
                            layout: &Self::texture_bind_group_layout(),
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(sampler),
                                },
                            ],
                        },
                    ))
                }
                None => SpriteType::Color,
            },
            transform_buffer: RendererGlobals::get().device.create_buffer_init(
                &BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(
                        &transforms.iter().map(Mat4::from).collect::<Vec<_>>(),
                    ),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                },
            ),
            transform_count: transforms.len() as u32,
            transforms: Dirty::new(transforms),
        }
    }

    #[must_use]
    pub fn new_quad_color(color: Color, transforms: Vec<Transform>) -> Self {
        let color = [color.r, color.g, color.b, color.a].map(|x| x as f32);

        Self::new_polygon(
            &[
                ColorVertex {
                    pos: vec3(-0.5, -0.5, 0.0),
                    color,
                },
                ColorVertex {
                    pos: vec3(0.5, -0.5, 0.0),
                    color,
                },
                ColorVertex {
                    pos: vec3(0.5, 0.5, 0.0),
                    color,
                },
                ColorVertex {
                    pos: vec3(-0.5, 0.5, 0.0),
                    color,
                },
            ],
            &[0, 1, 2, 0, 2, 3],
            None,
            transforms,
        )
    }

    #[must_use]
    pub fn new_quad_texture(
        view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
        source: Option<Rect>,
        transform: Vec<Transform>,
    ) -> Self {
        let source = source.unwrap_or_default();

        Self::new_polygon(
            &[
                TextureVertex {
                    pos: vec3(-0.5, -0.5, 0.0),
                    tex_coords: source.pos + vec2(0.0, source.size.y),
                },
                TextureVertex {
                    pos: vec3(0.5, -0.5, 0.0),
                    tex_coords: source.pos + source.size,
                },
                TextureVertex {
                    pos: vec3(0.5, 0.5, 0.0),
                    tex_coords: source.pos + vec2(source.size.x, 0.0),
                },
                TextureVertex {
                    pos: vec3(-0.5, 0.5, 0.0),
                    tex_coords: source.pos,
                },
            ],
            &[0, 1, 2, 0, 2, 3],
            Some((view, sampler)),
            transform,
        )
    }

    pub(crate) fn color_pipeline(
        projection_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let device = &RendererGlobals::get().device;
        let shader = device.create_shader_module(include_wgsl!("color.wgsl"));
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[projection_layout],
            push_constant_ranges: &[],
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("color pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "color_vertex",
                buffers: &[ColorVertex::desc(), Transform::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "color_fragment",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        })
    }
    pub(crate) fn texture_pipeline(
        projection_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let device = &RendererGlobals::get().device;
        let shader = device.create_shader_module(include_wgsl!("color.wgsl"));
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[projection_layout, &Self::texture_bind_group_layout()],
            push_constant_ranges: &[],
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("texture pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "texture_vertex",
                buffers: &[TextureVertex::desc(), Transform::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "texture_fragment",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        })
    }

    fn texture_bind_group_layout() -> wgpu::BindGroupLayout {
        RendererGlobals::get()
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
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
            })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorVertex {
    pub pos: Vec3,
    pub color: [f32; 4],
}

impl ColorVertex {
    pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<Vec3>() as u64,
                    shader_location: 5,
                },
            ],
        }
    }
}

impl Vertex for ColorVertex {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextureVertex {
    pub pos: Vec3,
    pub tex_coords: Vec2,
}

impl TextureVertex {
    pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: size_of::<Vec3>() as u64,
                    shader_location: 5,
                },
            ],
        }
    }
}

impl Vertex for TextureVertex {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub pos: Vec2,
    pub size: Vec2,
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            size: Vec2::ONE,
        }
    }
}
