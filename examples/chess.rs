use glam::{vec2, vec3};
use rengine::{
    input::InputState,
    renderer::Projection,
    sprite::{Color, Rect, Sprite},
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
        data.renderer.background = Color {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 1.0,
        };

        for sprite in (0..64).map(|index| {
            let x = index % 8;
            let y = index / 8;
            Sprite::new_color(
                if (x + y) % 2 == 0 {
                    Color::BLACK
                } else {
                    Color::WHITE
                },
                vec![Transform::translation(vec3(
                    x as f32 - 3.5,
                    y as f32 - 3.5,
                    -1.0,
                ))],
            )
        }) {
            data.renderer.sprites.insert(sprite);
        }

        let pieces = data
            .texture_manager
            .load(&texture::TextureSource::Memory(include_bytes!(
                "ChessPiecesArray.png"
            )));
        let sampler = data.texture_manager.linear_sampler();
        data.renderer.sprites.insert(Sprite::new_texture(
            &pieces,
            sampler,
            Some(Rect {
                pos: vec2(0.5, 0.0),
                size: vec2(1.0 / 6.0, 0.5),
            }),
            vec![Transform::default()],
        ));
    }

    fn update(&mut self, data: &mut rengine::GameData) {
        if data.input.is_key('q', InputState::Pressed) {
            data.exit();
        }
    }
}
