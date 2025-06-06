use super::{SecurityCheck, SecurityCheckContext};
use crate::UpgradeArgs;

pub struct UpgradeFunctionCheck;

impl UpgradeFunctionCheck {
    pub fn new() -> Self {
        UpgradeFunctionCheck
    }
}

impl SecurityCheck for UpgradeFunctionCheck {
    fn name(&self) -> &str {
        "Upgrade Function Check"
    }

    fn run(&self, _args: &UpgradeArgs, context: &mut SecurityCheckContext) -> Result<(), String> {
        if let Some(interface) = &context.contract_interface {
            // Check if the interface contains an upgrade function with the expected signature
            // The expected signature should include both the function name and the wasm_hash parameter
            if interface.contains("fn upgrade(")
                && interface.contains("new_wasm_hash: soroban_sdk::BytesN<32>")
            {
                println!("✅ Contract exposes an upgrade function with proper signature");
                Ok(())
            } else {
                Err("❌ Contract does not expose a proper upgrade function. Further upgradeability won't be possible.".to_string())
            }
        } else {
            Err("Contract interface information not available".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upgrade_function_check_pass() {
        let mut context = SecurityCheckContext::new();
        context.contract_interface = Some(
            r#"
        #[soroban_sdk::contractargs(name = "Args")]
        #[soroban_sdk::contractclient(name = "Client")]
        pub trait Contract {
            fn handle_upgrade(env: soroban_sdk::Env);
            fn version(env: soroban_sdk::Env) -> u32;
            fn upgrade(env: soroban_sdk::Env, new_wasm_hash: soroban_sdk::BytesN<32>);
        }
        "#
            .to_string(),
        );

        let check = UpgradeFunctionCheck::new();
        let result = check.run(
            &UpgradeArgs {
                id: "test".to_string(),
                wasm_hash: "test".to_string(),
                source: "test".to_string(),
                network: "test".to_string(),
                rpc_url: None,
                rpc_header: None,
                network_passphrase: None,
                fee: 100,
                is_view: false,
                instructions: None,
                build_only: false,
                send: None,
                cost: false,
                force: false,
                contract_args: vec![],
            },
            &mut context,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_upgrade_function_check_fail_no_function() {
        let mut context = SecurityCheckContext::new();
        context.contract_interface = Some(
            r#"
        #[soroban_sdk::contractargs(name = "Args")]
        #[soroban_sdk::contractclient(name = "Client")]
        pub trait Contract {
            fn handle_upgrade(env: soroban_sdk::Env);
            fn version(env: soroban_sdk::Env) -> u32;
        }
        "#
            .to_string(),
        );

        let check = UpgradeFunctionCheck::new();
        let result = check.run(
            &UpgradeArgs {
                id: "test".to_string(),
                wasm_hash: "test".to_string(),
                source: "test".to_string(),
                network: "test".to_string(),
                rpc_url: None,
                rpc_header: None,
                network_passphrase: None,
                fee: 100,
                is_view: false,
                instructions: None,
                build_only: false,
                send: None,
                cost: false,
                force: false,
                contract_args: vec![],
            },
            &mut context,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_upgrade_function_check_fail_wrong_signature() {
        let mut context = SecurityCheckContext::new();
        context.contract_interface = Some(
            r#"
        #[soroban_sdk::contractargs(name = "Args")]
        #[soroban_sdk::contractclient(name = "Client")]
        pub trait Contract {
            fn handle_upgrade(env: soroban_sdk::Env);
            fn version(env: soroban_sdk::Env) -> u32;
            fn upgrade(env: soroban_sdk::Env, hash: String); // Wrong parameter type
        }
        "#
            .to_string(),
        );

        let check = UpgradeFunctionCheck::new();
        let result = check.run(
            &UpgradeArgs {
                id: "test".to_string(),
                wasm_hash: "test".to_string(),
                source: "test".to_string(),
                network: "test".to_string(),
                rpc_url: None,
                rpc_header: None,
                network_passphrase: None,
                fee: 100,
                is_view: false,
                instructions: None,
                build_only: false,
                send: None,
                cost: false,
                force: false,
                contract_args: vec![],
            },
            &mut context,
        );

        assert!(result.is_err());
    }
}
