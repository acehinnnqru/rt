mod agrm;
mod cmd;
pub mod settings;

#[macro_use]
extern crate log;

fn init_logger() {
    let env = env_logger::Env::default().filter_or("AGRM_LOG", "info");

    env_logger::Builder::from_env(env)
        .format_timestamp(None)
        .format_target(false)
        .format_module_path(false)
        .init();
}

fn main() {
    init_logger();

    agrm::run()
}
