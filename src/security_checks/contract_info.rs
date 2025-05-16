use std::process::Command;
use crate::UpgradeArgs;
use super::SecurityCheckContext;

pub fn fetch_contract_interface(args: &UpgradeArgs, context: &mut SecurityCheckContext) -> Result<(), String> {
    println!("Fetching contract interface information...");
    
    // Construct the command to get contract interface
    let command = format!(
        "stellar contract info interface --wasm-hash {} --network {}",
        args.wasm_hash, args.network
    );
    
    // Execute the command
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &command])
            .output()
    } else {
        Command::new("sh")
            .args(["-c", &command])
            .output()
    };
    
    match output {
        Ok(output) => {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    context.contract_interface = Some(stdout);
                    Ok(())
                } else {
                    Err("Failed to parse contract interface output".to_string())
                }
            } else {
                if let Ok(stderr) = String::from_utf8(output.stderr) {
                    Err(format!("Failed to get contract interface: {}", stderr))
                } else {
                    Err("Failed to get contract interface".to_string())
                }
            }
        }
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
} 