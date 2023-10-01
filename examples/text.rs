use rengine::{
    renderer::Projection, sprite::Sprite, text::TextStyle, texture::TextureSource,
    transform::Transform, Game,
};
use wgpu::Color;

fn main() {
    rengine::run(Main);
}

struct Main;

impl Game for Main {
    fn init(&mut self, data: &mut rengine::GameData) {
        *data.renderer.projection = Projection::FixedHeight(800.0);
        data.renderer.window.set_title("Rengine Template");

        let font = data
            .text_manager
            .load_bytes(include_bytes!("BagnardSans.otf"))
            .unwrap();
        let texture = data.texture_manager.load(&TextureSource::Image(
            data.text_manager
                .make_image(&[TextStyle::new("Foo bar?", 24.0, font)], Color::WHITE)
                .unwrap(),
        ));
        data.renderer.sprites.insert(Sprite::new_texture(
            &texture,
            data.texture_manager.linear_sampler(),
            None,
            vec![Transform::texture_scale(&texture)],
        ));
    }
}
