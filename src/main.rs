mod cmd;
mod logging;
pub mod settings;

use cmd::agrm::Agrm;

fn main() {
    Agrm::new().unwrap().run()
}
