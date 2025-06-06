use super::{SecurityCheck, SecurityCheckContext};
use crate::UpgradeArgs;

pub struct ConstructorCheck;

impl ConstructorCheck {
    pub fn new() -> Self {
        ConstructorCheck
    }
}

impl SecurityCheck for ConstructorCheck {
    fn name(&self) -> &str {
        "Constructor Check"
    }

    fn run(&self, _args: &UpgradeArgs, context: &mut SecurityCheckContext) -> Result<(), String> {
        if let Some(interface) = &context.contract_interface {
            // Check if the interface contains a __constructor function
            if !interface.contains("fn __constructor(") {
                println!("✅ Contract does not have a __constructor function");
                Ok(())
            } else {
                Err("❌ Contract has a __constructor function, which might cause issues during upgrade.".to_string())
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
    fn test_constructor_check_pass() {
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

        let check = ConstructorCheck::new();
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
    fn test_constructor_check_fail() {
        let mut context = SecurityCheckContext::new();
        context.contract_interface = Some(
            r#"
        #[soroban_sdk::contractargs(name = "Args")]
        #[soroban_sdk::contractclient(name = "Client")]
        pub trait Contract {
            fn __constructor(env: soroban_sdk::Env, admin: soroban_sdk::Address);
            fn handle_upgrade(env: soroban_sdk::Env);
            fn version(env: soroban_sdk::Env) -> u32;
            fn upgrade(env: soroban_sdk::Env, new_wasm_hash: soroban_sdk::BytesN<32>);
        }
        "#
            .to_string(),
        );

        let check = ConstructorCheck::new();
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
