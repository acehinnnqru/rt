use crate::{cmd::clone, settings::Settings, logging};
use clap::{Parser, Subcommand};
use log::{debug, info};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[arg(short, long)]
    config: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(long)]
    verbose: bool,

    #[clap(skip)]
    settings: crate::settings::Settings,
}

impl Cli {
    pub fn run(mut self) -> ! {
        self.init_logger();
        self.get_settings();

        match self.command {
            Some(Commands::Clone(args)) => clone::run(args),
            None => todo!(),
        }
    }

    fn get_settings(&mut self) {
        if let Some(config) = &self.config {
            debug!("Using config: {}", config);
            self.settings = Settings::from(config.clone()).unwrap();
        } else {
            debug!("Using config from sources");
            self.settings = Settings::new().unwrap();
        }
        debug!("Settings: {:?}", self.settings);
    }

    fn init_logger(&self) {
        match self.verbose {
            true => logging::init(log::Level::Debug),
            false => logging::init(log::Level::Info),
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Clones repositories
    #[command(arg_required_else_help = true)]
    Clone(clone::CloneArgs),
}
