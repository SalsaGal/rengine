mod renderer;

use renderer::Renderer;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub fn run(mut game: impl Game + 'static) -> ! {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut renderer = pollster::block_on(Renderer::new(window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            game.update();
            renderer.window.request_redraw();
        }
        Event::RedrawRequested(..) => match renderer.render() {
            Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.window.inner_size()),
            Err(wgpu::SurfaceError::OutOfMemory) => panic!("SurfaceError: Out Of Memory!"),
            Err(e) => eprintln!("SurfaceError: {e}"),
            Ok(_) => {}
        },
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(size) => renderer.resize(size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                renderer.resize(*new_inner_size);
            }
            _ => {}
        },
        _ => {}
    })
}

pub trait Game {
    fn update(&mut self) {}
}
