use rengine::{
    renderer::Projection,
    sprite::{ColorSprite, TextureSprite},
    Game,
};

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

        data.renderer
            .texture_sprites
            .push(TextureSprite::new_quad());
    }
}
