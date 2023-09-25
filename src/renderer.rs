use winit::{dpi::PhysicalSize, window::Window};

pub struct Renderer {
    pub window: Window,
}

impl Renderer {
    pub(crate) async fn new(window: Window) -> Self {
        Self { window }
    }

    pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {}

    pub(crate) fn render(&self) -> Result<(), wgpu::SurfaceError> {
        Ok(())
    }
}
