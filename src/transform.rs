use std::mem::size_of;

use glam::{Mat4, Quat, Vec3};

use crate::texture::Texture;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    #[must_use]
    pub fn translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn texture_scale(texture: &Texture) -> Self {
        Self {
            scale: texture.size.as_vec2().extend(1.0),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn with_translation(self, translation: Vec3) -> Self {
        Self {
            translation,
            ..self
        }
    }

    #[must_use]
    pub fn with_rotation(self, rotation: Quat) -> Self {
        Self { rotation, ..self }
    }

    #[must_use]
    pub fn with_scale(self, scale: Vec3) -> Self {
        Self { scale, ..self }
    }

    #[must_use]
    pub fn with_texture_scale(self, texture: &Texture) -> Self {
        Self {
            scale: texture.size.as_vec2().extend(1.0),
            ..self
        }
    }

    #[must_use]
    pub fn and_then(mut self, f: impl Fn(&mut Self)) -> Self {
        f(&mut self);
        self
    }

    pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Mat4>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 4]>() as u64,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 8]>() as u64,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 12]>() as u64,
                    shader_location: 3,
                },
            ],
        }
    }
}

impl From<&Transform> for Mat4 {
    fn from(value: &Transform) -> Self {
        Self::from_scale_rotation_translation(value.scale, value.rotation, value.translation)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}
