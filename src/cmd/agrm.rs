use std::process;

use crate::{cmd::clone, settings::Settings};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = "A Git Repository Manager")]
pub(crate) struct Agrm {
    #[arg(short, long)]
    config: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(long)]
    verbose: bool,

    #[clap(skip)]
    settings: crate::settings::Settings,
}

impl Agrm {
    pub fn new() -> Option<Self> {
        match Self::try_parse() {
            Ok(cli) => Some(cli.init()),
            Err(e) => {
                error!("{}", e);
                process::exit(1)
            }
        }
    }

    fn init(self) -> Self {
        self.init_settings()
    }

    pub fn run(self) -> ! {
        match self.command {
            Some(Commands::Clone(args)) => clone::run(args),
            None => todo!(),
        }
    }

    fn init_settings(mut self) -> Self {
        if let Some(config) = &self.config {
            debug!("Using config: {}", config);
            self.settings = Settings::from(config.clone());
        } else {
            debug!("Using config from sources");
            self.settings = Settings::new();
        }
        debug!("Settings: {:?}", self.settings);

        self
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Clones repositories
    #[command(arg_required_else_help = true)]
    Clone(clone::CloneArgs),
}
