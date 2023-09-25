use rengine::{sprite::ColorSprite, Game};

fn main() {
    rengine::run(Main);
}

struct Main;

impl Game for Main {
    fn init(&mut self, data: rengine::GameData) {
        data.renderer
            .color_sprites
            .push(ColorSprite::new_quad(wgpu::Color::WHITE));
    }
}
