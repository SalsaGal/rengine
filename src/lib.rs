pub mod input;
pub mod renderer;
pub mod sprite;
pub mod texture;
pub mod transform;

use input::Input;
use renderer::Renderer;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub fn run(mut game: impl Game + 'static) -> ! {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut data = GameData {
        input: Input::new(),
        renderer: pollster::block_on(Renderer::new(window, renderer::Projection::FixedWidth(2.0))),
    };

    game.init(&mut data);

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            game.update(&mut data);
            data.input.update();
            data.renderer.window.request_redraw();
        }
        Event::RedrawRequested(..) => match data.renderer.render() {
            Err(wgpu::SurfaceError::Lost) => {
                data.renderer.resize(data.renderer.window.inner_size());
            }
            Err(wgpu::SurfaceError::OutOfMemory) => panic!("SurfaceError: Out Of Memory!"),
            Err(e) => eprintln!("SurfaceError: {e}"),
            Ok(_) => {}
        },
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::AxisMotion {
                device_id,
                axis,
                value,
            } => data.input.handle_axis(device_id, axis, value),
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::CursorMoved { position, .. } => data.input.handle_cursor(position),
            WindowEvent::KeyboardInput { input, .. } => data.input.handle_key(input),
            WindowEvent::MouseInput { state, button, .. } => {
                data.input.handle_button(button, state);
            }
            WindowEvent::Resized(size) => data.renderer.resize(size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                data.renderer.resize(*new_inner_size);
            }
            _ => {}
        },
        _ => {}
    })
}

pub struct GameData {
    pub input: Input,
    pub renderer: Renderer,
}

#[allow(unused)]
pub trait Game {
    fn init(&mut self, data: &mut GameData) {}
    fn update(&mut self, data: &mut GameData) {}
}
