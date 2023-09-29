use std::{path::Path, sync::Arc};

use fxhash::FxHashMap;
use image::RgbaImage;
use wgpu::{util::DeviceExt, TextureView};

use crate::renderer::RendererGlobals;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum TextureSource<'a> {
    Path(&'a Path),
    Image(RgbaImage),
    Memory(&'a [u8]),
}

#[derive(Debug, Default)]
pub struct TextureManager<'a> {
    textures: FxHashMap<TextureSource<'a>, Arc<TextureView>>,
    linear_sampler: Option<wgpu::Sampler>,
    nearest_sampler: Option<wgpu::Sampler>,
}

impl<'a> TextureManager<'a> {
    pub fn load(&mut self, source: TextureSource<'a>) -> Arc<TextureView> {
        self.textures
            .entry(source.clone()) // TODO cloning can't be good here
            .or_insert_with(|| {
                let image = match source {
                    TextureSource::Path(path) => image::open(path).unwrap().into_rgba8(),
                    TextureSource::Image(image) => image,
                    TextureSource::Memory(bytes) => {
                        image::load_from_memory(bytes).unwrap().into_rgba8()
                    }
                };

                Arc::new(
                    RendererGlobals::get()
                        .device
                        .create_texture_with_data(
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
                                usage: wgpu::TextureUsages::TEXTURE_BINDING
                                    | wgpu::TextureUsages::COPY_DST,
                                view_formats: &[],
                            },
                            &image,
                        )
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                )
            })
            .clone()
    }

    #[must_use]
    pub fn linear_sampler(&mut self) -> &wgpu::Sampler {
        self.linear_sampler.get_or_insert_with(|| {
            RendererGlobals::get()
                .device
                .create_sampler(&wgpu::SamplerDescriptor {
                    mag_filter: wgpu::FilterMode::Linear,
                    ..Default::default()
                })
        })
    }

    #[must_use]
    pub fn nearest_sampler(&mut self) -> &wgpu::Sampler {
        self.nearest_sampler.get_or_insert_with(|| {
            RendererGlobals::get()
                .device
                .create_sampler(&wgpu::SamplerDescriptor {
                    mag_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                })
        })
    }
}
