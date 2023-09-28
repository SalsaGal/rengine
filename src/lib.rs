pub mod camera;
pub mod input;
pub mod renderer;
pub mod sprite;
pub mod texture;
pub mod transform;

use std::time::{Duration, Instant};

use input::Input;
use renderer::Renderer;
use texture::TextureManager;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

/// Consumes the [`Game`] and runs it. Should be the last function in the main function.
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
        texture_manager: TextureManager::default(),
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

/// Common game state that is handed to the [`Game`] state during updates, and allowing control.
pub struct GameData<'a> {
    /// A manager for reading player input.
    pub input: Input,
    /// A manager to handle drawing graphics.
    pub renderer: Renderer,
    pub texture_manager: TextureManager<'a>,
    /// If `None` does nothing, but if set to `Some` then the program will exit, returning the `i32`.
    pub exit_code: Option<i32>,
    /// The time since the last update.
    pub delta_time: Duration,
    /// The time that the game began running.
    pub start_time: Instant,
}

impl GameData<'_> {
    /// Shorthand to set `exit_code` to 0.
    pub fn exit(&mut self) {
        self.exit_code = Some(0);
    }
}

/// This trait must be implemented on a `struct` that handles the control flow of the game.
#[allow(unused)]
pub trait Game {
    /// Called at the beginning of the game, used for setting up models and anything else that requires the engine state
    /// have been created but nothing else.
    fn init(&mut self, data: &mut GameData) {}
    /// Called every frame.
    fn update(&mut self, data: &mut GameData) {}
}
