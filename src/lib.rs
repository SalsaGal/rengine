pub mod camera;
pub mod input;
pub mod renderer;
pub mod sprite;
pub mod texture;
pub mod transform;

use std::time::{Duration, Instant};

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
        renderer: pollster::block_on(Renderer::new(
            window,
            camera::Camera::default(),
            renderer::Projection::FixedWidth(2.0),
        )),
        exit_code: None,
        delta_time: Duration::default(),
        start_time: Instant::now(),
    };

    game.init(&mut data);

    let mut last_update = Instant::now();
    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            data.delta_time = Instant::now().duration_since(last_update);
            game.update(&mut data);
            if let Some(code) = data.exit_code {
                *control_flow = ControlFlow::ExitWithCode(code);
            }
            data.input.update();
            data.renderer.window.request_redraw();
            last_update = Instant::now();
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
    pub exit_code: Option<i32>,
    pub delta_time: Duration,
    pub start_time: Instant,
}

impl GameData {
    pub fn exit(&mut self) {
        self.exit_code = Some(0);
    }
}

#[allow(unused)]
pub trait Game {
    fn init(&mut self, data: &mut GameData) {}
    fn update(&mut self, data: &mut GameData) {}
}
