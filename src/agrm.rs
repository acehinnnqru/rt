use crate::{cmd::cli::Cli, settings::Settings};

#[derive(Default)]
pub struct Agrm {
    cli: Option<Cli>,
    settings: Option<Settings>,
}

impl Agrm {
    pub fn init() -> Self {
        let mut agrm = Self::default();
        agrm.init_logger();
        let c = Cli::new();
        agrm.cli = c;

        agrm.init_settings()
    }

    pub fn run(self) -> ! {
        self.cli.unwrap().run();
    }

    fn init_logger(&self) {
        let env = env_logger::Env::default().filter_or("AGRM_LOG", "info");

        env_logger::Builder::from_env(env)
            .format_timestamp(None)
            .format_target(false)
            .format_module_path(false)
            .init();
    }

    fn init_settings(mut self) -> Self {
        let config = self.cli.as_ref().unwrap().config.clone();
        let s = match config {
            Some(config) => Settings::from_file(config),
            None => Settings::from_configs(),
        };
        match s {
            Ok(s) => self.settings = Some(s),
            Err(e) => {
                error!("Error while loading settings: {}", e);
                warn!("Using default settings");
                self.settings = Some(Settings::default());
            }
        };

        debug!("Settings: {:?}", self.settings);

        self
    }
}
