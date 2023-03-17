use crate::{cmd::cli::Cli, settings::Settings};

#[derive(Default)]
pub struct Agrm {
    cli: Option<Cli>,
    settings: Option<Settings>,
}

impl Agrm {
    pub fn new() -> Self {
        Self {
            cli: Cli::new(),
            settings: None,
        }
    }

    pub fn run(self) -> ! {
        self.cli.unwrap().run();
    }
}
