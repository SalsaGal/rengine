use rengine::Game;

fn main() {
    rengine::run(Main);
}

struct Main;

impl Game for Main {}
