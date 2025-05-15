use clap::{Parser, Subcommand};
use std::process::Command;

mod security_checks;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct UpgraderCli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Upgrade a Stellar smart contract
    Upgrade(UpgradeArgs),
}

#[derive(Parser)]
pub struct UpgradeArgs {
    /// The contract ID to upgrade
    #[arg(long)]
    pub id: String,
    
    /// The new WASM hash for the upgrade
    #[arg(long = "wasm-hash")]
    pub wasm_hash: String,
    
    /// Source account to pay for the upgrade
    #[arg(long, default_value = "alice")]
    pub source: String,
    
    /// Network to use (testnet, futurenet, mainnet)
    #[arg(long, default_value = "testnet")]
    pub network: String,
}

/// Execute a shell command and return the result
fn execute_command(command: &str) -> Result<(), String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", command])
            .output()
    } else {
        Command::new("sh")
            .args(["-c", command])
            .output()
    };

    match output {
        Ok(output) => {
            if output.status.success() {
                // Print stdout
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if !stdout.trim().is_empty() {
                        println!("{}", stdout);
                    }
                }
                Ok(())
            } else {
                // Print stderr in case of error
                if let Ok(stderr) = String::from_utf8(output.stderr) {
                    Err(stderr)
                } else {
                    Err("Command failed with unknown error".to_string())
                }
            }
        }
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}

/// Generate the actual upgrade command
pub fn generate_upgrade_command(args: &UpgradeArgs) -> String {
    format!(
        "stellar contract invoke \
        --id {} \
        --source {} \
        --network {} \
        -- \
        upgrade \
        --new_wasm_hash {}",
        args.id, args.source, args.network, args.wasm_hash
    )
}

/// Run the upgrade command after security checks
pub fn run_upgrade(args: &UpgradeArgs) -> Result<(), String> {
    // Perform security checks using the modular system
    security_checks::run_all_checks(args)?;
    
    // Generate the upgrade command
    let command = generate_upgrade_command(args);
    
    // Display the command to be executed
    println!("Executing: {}", command);
    
    // Actually execute the command
    execute_command(&command)
} 