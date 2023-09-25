use std::path::Path;

use image::{ImageError, RgbaImage};
use wgpu::util::DeviceExt;

use crate::renderer::RendererGlobals;

pub fn from_image(image: &RgbaImage) -> Result<wgpu::TextureView, ImageError> {
    let texture = RendererGlobals::get().device.create_texture_with_data(
        &RendererGlobals::get().queue,
        &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        &image,
    );

    Ok(texture.create_view(&wgpu::TextureViewDescriptor::default()))
}

pub fn from_memory(data: &[u8]) -> Result<wgpu::TextureView, ImageError> {
    from_image(&image::load_from_memory(data)?.into_rgba8())
}

pub fn from_path<P: AsRef<Path>>(path: P) -> Result<wgpu::TextureView, ImageError> {
    from_image(&image::open(path)?.into_rgba8())
}

#[must_use]
pub fn linear_sampler() -> wgpu::Sampler {
    RendererGlobals::get()
        .device
        .create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        })
}

#[must_use]
pub fn nearest_sampler() -> wgpu::Sampler {
    RendererGlobals::get()
        .device
        .create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
}
