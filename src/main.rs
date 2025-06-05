use clap::Parser;
use stellar_upgrader_plugin::{run_upgrade, Commands, UpgraderCli};

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
