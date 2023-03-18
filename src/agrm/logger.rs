const ENV_AGRM_LOG: &str = "AGRM_LOG";

pub fn init() {
    let env = env_logger::Env::default().filter_or(ENV_AGRM_LOG, "info");

    env_logger::Builder::from_env(env)
        .format_timestamp(None)
        .format_target(false)
        .format_module_path(false)
        .init();
}

