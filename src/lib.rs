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

    /// RPC server endpoint
    #[arg(long)]
    pub rpc_url: Option<String>,

    /// RPC Header(s) to include in requests to the RPC provider
    #[arg(long)]
    pub rpc_header: Option<Vec<String>>,

    /// Network passphrase to sign the transaction
    #[arg(long)]
    pub network_passphrase: Option<String>,

    /// Fee amount for transaction, in stroops (1 stroop = 0.0000001 XLM)
    #[arg(long, default_value = "100")]
    pub fee: u32,

    /// Whether to only simulate the transaction
    #[arg(long)]
    pub is_view: bool,

    /// Number of instructions to simulate
    #[arg(long)]
    pub instructions: Option<u32>,

    /// Only build the transaction and output base64 XDR
    #[arg(long)]
    pub build_only: bool,

    /// Whether to send the transaction (yes, no, default)
    #[arg(long, default_value = "default")]
    pub send: Option<String>,

    /// Output the cost execution to stderr
    #[arg(long)]
    pub cost: bool,
    
    /// Additional contract function arguments
    #[arg(last = true)]
    pub contract_args: Vec<String>,
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
    let mut command = format!(
        "stellar contract invoke \
        --id {} \
        --source {} \
        --network {}",
        args.id, args.source, args.network
    );

    // Add optional parameters
    if let Some(rpc_url) = &args.rpc_url {
        command.push_str(&format!(" --rpc-url {}", rpc_url));
    }

    if let Some(headers) = &args.rpc_header {
        for header in headers {
            command.push_str(&format!(" --rpc-header {}", header));
        }
    }

    if let Some(passphrase) = &args.network_passphrase {
        command.push_str(&format!(" --network-passphrase {}", passphrase));
    }

    if args.fee != 100 {
        command.push_str(&format!(" --fee {}", args.fee));
    }

    if args.is_view {
        command.push_str(" --is-view");
    }

    if let Some(instr) = args.instructions {
        command.push_str(&format!(" --instructions {}", instr));
    }

    if args.build_only {
        command.push_str(" --build-only");
    }

    if let Some(send) = &args.send {
        command.push_str(&format!(" --send {}", send));
    }

    if args.cost {
        command.push_str(" --cost");
    }

    // Add the contract function and args
    command.push_str(" -- upgrade");
    command.push_str(&format!(" --new_wasm_hash {}", args.wasm_hash));
    
    // Add any additional contract args
    if !args.contract_args.is_empty() {
        for arg in &args.contract_args {
            command.push_str(&format!(" {}", arg));
        }
    }

    command
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