use glam::{vec2, vec3};
use rengine::{
    input::InputState,
    renderer::Projection,
    sprite::{Rect, Sprite},
    texture,
    transform::Transform,
    Game,
};

fn main() {
    rengine::run(Chess);
}

struct Chess;

impl Game for Chess {
    fn init(&mut self, data: &mut rengine::GameData) {
        *data.renderer.projection = Projection::FixedMinimum(10.0, 10.0);
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
                    wgpu::Color::BLACK
                } else {
                    wgpu::Color::WHITE
                },
                &[Transform::translation(vec3(
                    x as f32 - 3.5,
                    y as f32 - 3.5,
                    -1.0,
                ))],
            )
        }));

        let pieces = texture::from_memory(include_bytes!("ChessPiecesArray.png")).unwrap();
        let sampler = texture::linear_sampler();
        data.renderer.sprites.push(Sprite::new_quad_texture(
            &pieces,
            &sampler,
            Some(Rect {
                pos: vec2(0.5, 0.0),
                size: vec2(1.0 / 6.0, 0.5),
            }),
            &[Transform::default()],
        ));
    }

    fn update(&mut self, data: &mut rengine::GameData) {
        if data.input.is_key('q', InputState::Pressed) {
            data.exit();
        }
    }
}
