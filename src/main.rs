mod cmd;
mod global;
pub mod settings;

#[macro_use]
extern crate log;
use cmd::agrm::Agrm;

fn main() {
    init();
    Agrm::new().unwrap().run()
}

fn init() {
    env_logger::init_from_env(env_logger::Env::default().filter(global::ENV_AGRM_LOG));
}
