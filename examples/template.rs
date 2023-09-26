use rengine::{renderer::Projection, sprite::TextureSprite, texture, Game};

fn main() {
    rengine::run(Main);
}

struct Main;

impl Game for Main {
    fn init(&mut self, data: rengine::GameData) {
        // data.renderer
        //     .color_sprites
        //     .push(ColorSprite::new_quad(wgpu::Color::WHITE));

        *data.renderer.projection = Projection::FixedHeight(1.0);
        data.renderer.window.set_title("Rengine Template");

        data.renderer.texture_sprites.push(TextureSprite::new_quad(
            &texture::from_memory(include_bytes!("test.png")).unwrap(),
            &texture::linear_sampler(),
        ));
    }
}
