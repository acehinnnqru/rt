mod agrm;
mod cmd;
mod global;
pub mod settings;

#[macro_use]
extern crate log;

fn main() {
    agrm::Agrm::new().run()
}
