use glam::Vec3;
use rengine::{renderer::Projection, sprite::Sprite, texture, transform::Transform, Game};

fn main() {
    rengine::run(Main);
}

struct Main;

impl Game for Main {
    fn init(&mut self, data: &mut rengine::GameData) {
        *data.renderer.projection = Projection::FixedHeight(2.0);
        data.renderer.window.set_title("Rengine Template");

        data.renderer.sprites.push(Sprite::new_quad_texture(
            &texture::from_memory(include_bytes!("test.png")).unwrap(),
            &texture::linear_sampler(),
            &[
                Transform {
                    translation: Vec3::NEG_X,
                    ..Default::default()
                },
                Transform {
                    translation: Vec3::X,
                    ..Default::default()
                },
            ],
        ));
    }
}
