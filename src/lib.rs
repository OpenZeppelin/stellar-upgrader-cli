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
    
    /// Force upgrade and skip security checks
    #[arg(long)]
    pub force: bool,
    
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

/// Ask for user confirmation when using --force flag
fn confirm_force_upgrade() -> Result<bool, String> {
    print!("Are you sure you want to proceed without security checks? (y/N): ");
    std::io::Write::flush(&mut std::io::stdout()).map_err(|e| format!("Failed to flush stdout: {}", e))?;
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).map_err(|e| format!("Failed to read user input: {}", e))?;
    
    let input = input.trim().to_lowercase();
    Ok(input == "y" || input == "yes")
}

/// Check if user confirms force upgrade (testable version)
fn check_force_confirmation(input: Option<&str>) -> Result<bool, String> {
    match input {
        Some(user_input) => {
            let input = user_input.trim().to_lowercase();
            Ok(input == "y" || input == "yes")
        }
        None => confirm_force_upgrade(),
    }
}

/// Run the upgrade command after security checks
pub fn run_upgrade(args: &UpgradeArgs) -> Result<(), String> {
    run_upgrade_with_input(args, None)
}

/// Run the upgrade command with optional input (for testing)
pub fn run_upgrade_with_input(args: &UpgradeArgs, force_input: Option<&str>) -> Result<(), String> {
    // Conditionally perform security checks based on --force flag
    if args.force {
        println!("⚠️  WARNING: Security checks are being skipped due to --force flag!");
        println!("⚠️  This may result in upgrade failures or loss of upgradeability.");
        println!("⚠️  Proceed with caution!\n");
        
        if !check_force_confirmation(force_input)? {
            return Err("Upgrade cancelled by user".to_string());
        }
        println!();
    } else {
        // Perform security checks using the modular system
        security_checks::run_all_checks(args)?;
    }
    
    // Generate the upgrade command
    let command = generate_upgrade_command(args);
    
    // Display the command to be executed
    println!("Executing: {}", command);
    
    // Actually execute the command
    execute_command(&command)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_args_with_force(force: bool) -> UpgradeArgs {
        UpgradeArgs {
            id: "test_contract".to_string(),
            wasm_hash: "test_hash".to_string(),
            source: "alice".to_string(),
            network: "testnet".to_string(),
            rpc_url: None,
            rpc_header: None,
            network_passphrase: None,
            fee: 100,
            is_view: false,
            instructions: None,
            build_only: true, // Use build_only to avoid actually executing commands
            send: None,
            cost: false,
            force,
            contract_args: vec![],
        }
    }

    #[test]
    fn test_force_flag_with_yes_confirmation() {
        let args = create_test_args_with_force(true);
        
        // Test with "y" input
        let result = run_upgrade_with_input(&args, Some("y"));
        // This will fail because we don't have stellar CLI, but it should get past the confirmation
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        // Should fail with contract not found or command execution error, not confirmation error
        assert!(error_msg.contains("contract not found") || error_msg.contains("Failed to execute command"));
        
        // Test with "yes" input
        let result = run_upgrade_with_input(&args, Some("yes"));
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("contract not found") || error_msg.contains("Failed to execute command"));
        
        // Test with "Y" input (uppercase)
        let result = run_upgrade_with_input(&args, Some("Y"));
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("contract not found") || error_msg.contains("Failed to execute command"));
        
        // Test with "YES" input (uppercase)
        let result = run_upgrade_with_input(&args, Some("YES"));
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("contract not found") || error_msg.contains("Failed to execute command"));
    }

    #[test]
    fn test_force_flag_with_no_confirmation() {
        let args = create_test_args_with_force(true);
        
        // Test with "n" input
        let result = run_upgrade_with_input(&args, Some("n"));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Upgrade cancelled by user");
        
        // Test with "no" input
        let result = run_upgrade_with_input(&args, Some("no"));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Upgrade cancelled by user");
        
        // Test with empty input (default is no)
        let result = run_upgrade_with_input(&args, Some(""));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Upgrade cancelled by user");
        
        // Test with random input (default is no)
        let result = run_upgrade_with_input(&args, Some("maybe"));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Upgrade cancelled by user");
    }

    #[test]
    fn test_no_force_flag_runs_security_checks() {
        let args = create_test_args_with_force(false);
        
        // This should try to run security checks and fail because we don't have a real contract
        let result = run_upgrade_with_input(&args, None);
        assert!(result.is_err());
        // Should fail during security checks, not during command execution
        let error_msg = result.unwrap_err();
        // The test fails because of invalid wasm hash or contract interface issues
        assert!(error_msg.contains("Failed to get contract interface") || 
                error_msg.contains("Contract interface information not available") ||
                error_msg.contains("invalid") ||
                error_msg.contains("Failed to execute command"));
    }

    #[test]
    fn test_generate_upgrade_command() {
        let args = UpgradeArgs {
            id: "test_contract".to_string(),
            wasm_hash: "abc123".to_string(),
            source: "alice".to_string(),
            network: "testnet".to_string(),
            rpc_url: Some("https://test.com".to_string()),
            rpc_header: Some(vec!["Auth: Bearer token".to_string()]),
            network_passphrase: None,
            fee: 200,
            is_view: true,
            instructions: Some(50000),
            build_only: false,
            send: Some("yes".to_string()),
            cost: true,
            force: false, // force flag shouldn't affect command generation
            contract_args: vec!["--extra".to_string(), "arg".to_string()],
        };

        let command = generate_upgrade_command(&args);
        
        assert!(command.contains("stellar contract invoke"));
        assert!(command.contains("--id test_contract"));
        assert!(command.contains("--source alice"));
        assert!(command.contains("--network testnet"));
        assert!(command.contains("--rpc-url https://test.com"));
        assert!(command.contains("--rpc-header Auth: Bearer token"));
        assert!(command.contains("--fee 200"));
        assert!(command.contains("--is-view"));
        assert!(command.contains("--instructions 50000"));
        assert!(command.contains("--send yes"));
        assert!(command.contains("--cost"));
        assert!(command.contains("-- upgrade"));
        assert!(command.contains("--new_wasm_hash abc123"));
        assert!(command.contains("--extra arg"));
    }

    #[test]
    fn test_check_force_confirmation_function() {
        // Test positive confirmations
        assert!(check_force_confirmation(Some("y")).unwrap());
        assert!(check_force_confirmation(Some("yes")).unwrap());
        assert!(check_force_confirmation(Some("Y")).unwrap());
        assert!(check_force_confirmation(Some("YES")).unwrap());
        assert!(check_force_confirmation(Some(" y ")).unwrap()); // with whitespace
        assert!(check_force_confirmation(Some(" yes ")).unwrap()); // with whitespace
        
        // Test negative confirmations
        assert!(!check_force_confirmation(Some("n")).unwrap());
        assert!(!check_force_confirmation(Some("no")).unwrap());
        assert!(!check_force_confirmation(Some("")).unwrap());
        assert!(!check_force_confirmation(Some("maybe")).unwrap());
        assert!(!check_force_confirmation(Some("nope")).unwrap());
        assert!(!check_force_confirmation(Some("false")).unwrap());
    }
} 