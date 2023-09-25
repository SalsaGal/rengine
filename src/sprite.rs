use std::mem::size_of;

use glam::{vec2, vec3, Vec2, Vec3};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::renderer::RendererGlobals;

pub struct ColorSprite {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) index_count: u32,
}

impl ColorSprite {
    #[must_use] pub fn new_polygon(vertices: &[ColorVertex], indices: &[u16]) -> Self {
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
        }
    }

    #[must_use] pub fn new_quad(color: wgpu::Color) -> Self {
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
        )
    }

    pub(crate) fn pipeline(projection_layout: &wgpu::BindGroupLayout) -> wgpu::RenderPipeline {
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
                buffers: &[ColorVertex::desc()],
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
            depth_stencil: None,
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
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<Vec3>() as u64,
                    shader_location: 1,
                },
            ],
        }
    }
}

pub struct TextureSprite {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) index_count: u32,
    pub(crate) texture_bind_group: wgpu::BindGroup,
}

impl TextureSprite {
    #[must_use] pub fn new_polygon(
        vertices: &[TextureVertex],
        indices: &[u16],
        texture: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
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
            texture_bind_group: RendererGlobals::get().device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &Self::texture_bind_group_layout(),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(texture),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                    ],
                },
            ),
        }
    }

    #[must_use] pub fn new_quad(texture: &wgpu::TextureView, sampler: &wgpu::Sampler) -> Self {
        Self::new_polygon(
            &[
                TextureVertex {
                    pos: vec3(-0.5, -0.5, 0.0),
                    tex_coords: vec2(0.0, 1.0),
                },
                TextureVertex {
                    pos: vec3(0.5, -0.5, 0.0),
                    tex_coords: vec2(1.0, 1.0),
                },
                TextureVertex {
                    pos: vec3(0.5, 0.5, 0.0),
                    tex_coords: vec2(1.0, 0.0),
                },
                TextureVertex {
                    pos: vec3(-0.5, 0.5, 0.0),
                    tex_coords: vec2(0.0, 0.0),
                },
            ],
            &[0, 1, 2, 0, 2, 3],
            texture,
            sampler,
        )
    }

    pub(crate) fn pipeline(projection_layout: &wgpu::BindGroupLayout) -> wgpu::RenderPipeline {
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
                buffers: &[TextureVertex::desc()],
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
            depth_stencil: None,
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
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: size_of::<Vec3>() as u64,
                    shader_location: 1,
                },
            ],
        }
    }
}
