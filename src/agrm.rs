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
        match config {
            Some(config) => {
                self.settings = Some(Settings::from(config));
            }
            None => {
                self.settings = Some(Settings::new());
            }
        }

        self
    }
}
