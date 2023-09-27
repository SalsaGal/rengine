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

    let mut input_handler = Input::new();
    let mut renderer =
        pollster::block_on(Renderer::new(window, renderer::Projection::FixedWidth(2.0)));

    game.init(GameData {
        renderer: &mut renderer,
        input: &input_handler,
    });

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            game.update(GameData {
                renderer: &mut renderer,
                input: &input_handler,
            });
            input_handler.update();
            renderer.window.request_redraw();
        }
        Event::RedrawRequested(..) => match renderer.render() {
            Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.window.inner_size()),
            Err(wgpu::SurfaceError::OutOfMemory) => panic!("SurfaceError: Out Of Memory!"),
            Err(e) => eprintln!("SurfaceError: {e}"),
            Ok(_) => {}
        },
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::AxisMotion {
                device_id,
                axis,
                value,
            } => input_handler.handle_axis(device_id, axis, value),
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::CursorMoved { position, .. } => input_handler.handle_cursor(position),
            WindowEvent::KeyboardInput { input, .. } => input_handler.handle_key(input),
            WindowEvent::MouseInput { state, button, .. } => {
                input_handler.handle_button(button, state)
            }
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
    pub input: &'a Input,
}

#[allow(unused)]
pub trait Game {
    fn init(&mut self, data: GameData) {}
    fn update(&mut self, data: GameData) {}
}
