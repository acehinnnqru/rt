mod agrm;
mod cmd;
mod global;
pub mod settings;

fn main() {
    agrm::Agrm::init().run()
}
