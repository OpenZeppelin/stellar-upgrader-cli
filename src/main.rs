use clap::Parser;
use stellar_upgrader_plugin::{Commands, UpgraderCli, run_upgrade};

fn main() {
    let cli = UpgraderCli::parse();

    match cli.command {
        Commands::Upgrade(args) => {
            if let Err(err) = run_upgrade(&args) {
                eprintln!("Error: {}", err);
                std::process::exit(1);
            }
        }
    }
} 