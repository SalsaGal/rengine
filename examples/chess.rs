use glam::vec3;
use rengine::{
    input::InputState, renderer::Projection, sprite::Sprite, transform::Transform, Game,
};

fn main() {
    rengine::run(Chess);
}

struct Chess;

impl Game for Chess {
    fn init(&mut self, data: &mut rengine::GameData) {
        *data.renderer.projection = Projection::FixedHeight(10.0);
        data.renderer.background = wgpu::Color {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 1.0,
        };

        data.renderer.sprites.extend((0..64).map(|index| {
            let x = index % 8;
            let y = index / 8;
            Sprite::new_quad_color(
                if (x + y) % 2 == 0 {
                    wgpu::Color::WHITE
                } else {
                    wgpu::Color::BLACK
                },
                &[Transform {
                    translation: vec3(x as f32 - 3.5, y as f32 - 3.5, 0.0),
                    ..Default::default()
                }],
            )
        }));
    }

    fn update(&mut self, data: &mut rengine::GameData) {
        if data.input.is_key('q', InputState::Pressed) {
            println!("Quit");
        }
    }
}
