use crate::{cmd, config, consts::AGRM, repository::Repository};
use clap::CommandFactory;

fn long_about() -> String {
    "
`agrm` will clone a bare repository into a directory which named as `{root}/{git platform}/{namespace}/{name}/.bare`.

The params in the directory name:

- `root` is the root from config file `{$HOME}/.agrm.toml`
- `git platform` means the target platform in the provided repository url/ssh.
- `namespace` and `name` are also extract from the provided repository url/ssh.".to_string()
}

#[derive(clap::Parser)]
#[command(name = AGRM)]
#[command(bin_name = AGRM)]
#[command(version)]
#[command(about, long_about = long_about())]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    #[command(alias = "c")]
    Clone {
        #[arg(value_parser = clap::value_parser!(Repository))]
        repo: Repository,
    },
    #[command(alias = "v")]
    Version,
    #[command(alias = "conf")]
    Config,
}

pub fn main(args: Args) {
    match args.command {
        Commands::Clone { repo } => {
            println!("{:?}", config::config());

            println!("\nparsed repository: {:?}", repo);

            if config::root().trim().is_empty() {
                unreachable!("\ninvalide root setting")
            }

            cmd::clone(config::root(), &repo)
        }
        Commands::Version => {
            println!("agrm version: {}", Args::command().get_version().unwrap());
        }
        Commands::Config => cmd::print_config(),
    }
}
