use glam::Vec3;
use rengine::{
    renderer::Projection, sprite::Sprite, texture::TextureSource, transform::Transform, Game,
};

fn main() {
    rengine::run(Main);
}

struct Main;

impl Game for Main {
    fn init(&mut self, data: &mut rengine::GameData) {
        *data.renderer.projection = Projection::FixedHeight(2.0);
        data.renderer.window.set_title("Rengine Template");

        data.renderer.sprites.insert(Sprite::new_texture(
            &data
                .texture_manager
                .load(&TextureSource::Memory(include_bytes!("test.png"))),
            data.texture_manager.linear_sampler(),
            None,
            vec![
                Transform::translation(Vec3::NEG_X),
                Transform::translation(Vec3::X),
            ],
        ));
    }
}
