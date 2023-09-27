pub mod renderer;
pub mod sprite;
pub mod texture;
pub mod transform;

use renderer::Renderer;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub fn run(mut game: impl Game + 'static) -> ! {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut renderer =
        pollster::block_on(Renderer::new(window, renderer::Projection::FixedWidth(2.0)));

    game.init(GameData {
        renderer: &mut renderer,
    });

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            game.update(GameData {
                renderer: &mut renderer,
            });
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

pub struct GameData<'a> {
    pub renderer: &'a mut Renderer,
}

#[allow(unused)]
pub trait Game {
    fn init(&mut self, data: GameData) {}
    fn update(&mut self, data: GameData) {}
}
