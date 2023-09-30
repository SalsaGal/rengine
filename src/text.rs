//! Simple text rendering.

use std::{fs::File, io::Read, path::Path};

use anyhow::{anyhow, Result};
pub use fontdue::layout::*;
use fontdue::{Font, FontSettings};
use fxhash::FxHashMap;
use glam::UVec2;
use image::RgbaImage;
use slab::Slab;

/// A manager of fonts and text systems.
pub struct TextManager {
    pub fonts: Slab<Font>,
}

impl TextManager {
    pub(crate) fn new() -> Self {
        Self { fonts: Slab::new() }
    }

    /// Loads a font from file contents.
    pub fn load_bytes(&mut self, contents: &[u8]) -> Result<usize> {
        let font = Font::from_bytes(contents, FontSettings::default()).map_err(|e| anyhow!(e))?;
        Ok(self.fonts.insert(font))
    }

    /// Loads a font from a file and returns its index.
    pub fn load_font<P: AsRef<Path>>(&mut self, path: P) -> Result<usize> {
        let mut file = File::open(path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        self.load_bytes(&contents)
    }

    /// Render text to an image.
    #[must_use]
    pub fn make_image(
        &self,
        layout: &Layout,
        color: wgpu::Color,
        min_size: Option<UVec2>,
    ) -> Option<RgbaImage> {
        let min_size = min_size.unwrap_or(UVec2::ZERO);
        let mut pixels = FxHashMap::default();
        let mut max_x = 0;
        let mut max_y = 0;
        for c in layout.glyphs() {
            let font = self.fonts.get(c.font_index)?;
            let (metrics, bitmap) = font.rasterize(c.parent, c.key.px);
            for x in 0..metrics.width {
                for y in 0..metrics.height {
                    let value = bitmap[x + y * metrics.width];
                    let x = x as u32 + c.x as u32;
                    let y = y as u32 + c.y as u32;
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                    pixels.insert((x, y), value);
                }
            }
        }

        // Add 2 instead of one for padding so that linear interpolation doesn't cause artifacts at
        // the top of the texture
        let mut image = RgbaImage::new(min_size.x.max(max_x) + 1, min_size.y.max(max_y) + 1);
        let color = [
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
        ];
        for ((x, y), value) in pixels {
            image.put_pixel(x, y, [color[0], color[1], color[2], value].into());
        }
        Some(image)
    }
}
