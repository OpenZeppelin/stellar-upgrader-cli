#[cfg(test)]
mod integration_tests {
    use crate::security_checks::{SecurityCheckContext, SecurityCheck};
    use crate::UpgradeArgs;
    use crate::security_checks::constructor_check::ConstructorCheck;
    use crate::security_checks::upgrade_function_check::UpgradeFunctionCheck;
    
    fn create_test_args() -> UpgradeArgs {
        UpgradeArgs {
            id: "test_id".to_string(),
            wasm_hash: "test_hash".to_string(),
            source: "test_source".to_string(),
            network: "testnet".to_string(),
            rpc_url: None,
            rpc_header: None,
            network_passphrase: None,
            fee: 100,
            is_view: false,
            instructions: None,
            build_only: false,
            send: None,
            cost: false,
            contract_args: vec![],
        }
    }
    
    #[test]
    fn test_all_checks_pass() {
        let mut context = SecurityCheckContext::new();
        // Define a contract interface with no constructor and with proper upgrade function
        context.contract_interface = Some(r#"
        #[soroban_sdk::contractargs(name = "Args")]
        #[soroban_sdk::contractclient(name = "Client")]
        pub trait Contract {
            fn handle_upgrade(env: soroban_sdk::Env);
            fn version(env: soroban_sdk::Env) -> u32;
            fn upgrade(env: soroban_sdk::Env, new_wasm_hash: soroban_sdk::BytesN<32>);
        }
        "#.to_string());
        
        let args = create_test_args();
        
        // Run constructor check
        let constructor_check = ConstructorCheck::new();
        let constructor_result = constructor_check.run(&args, &mut context);
        assert!(constructor_result.is_ok(), "Constructor check failed: {:?}", constructor_result);
        
        // Run upgrade function check
        let upgrade_check = UpgradeFunctionCheck::new();
        let upgrade_result = upgrade_check.run(&args, &mut context);
        assert!(upgrade_result.is_ok(), "Upgrade function check failed: {:?}", upgrade_result);
    }
    
    #[test]
    fn test_constructor_check_fails() {
        let mut context = SecurityCheckContext::new();
        // Define a contract interface WITH constructor
        context.contract_interface = Some(r#"
        #[soroban_sdk::contractargs(name = "Args")]
        #[soroban_sdk::contractclient(name = "Client")]
        pub trait Contract {
            fn __constructor(env: soroban_sdk::Env, admin: soroban_sdk::Address);
            fn handle_upgrade(env: soroban_sdk::Env);
            fn version(env: soroban_sdk::Env) -> u32;
            fn upgrade(env: soroban_sdk::Env, new_wasm_hash: soroban_sdk::BytesN<32>);
        }
        "#.to_string());
        
        let args = create_test_args();
        
        // Run constructor check - should fail
        let constructor_check = ConstructorCheck::new();
        let constructor_result = constructor_check.run(&args, &mut context);
        assert!(constructor_result.is_err());
        
        // Run upgrade function check - should pass
        let upgrade_check = UpgradeFunctionCheck::new();
        let upgrade_result = upgrade_check.run(&args, &mut context);
        assert!(upgrade_result.is_ok());
    }
    
    #[test]
    fn test_upgrade_function_check_fails() {
        let mut context = SecurityCheckContext::new();
        // Define a contract interface without constructor but missing upgrade function
        context.contract_interface = Some(r#"
        #[soroban_sdk::contractargs(name = "Args")]
        #[soroban_sdk::contractclient(name = "Client")]
        pub trait Contract {
            fn handle_upgrade(env: soroban_sdk::Env);
            fn version(env: soroban_sdk::Env) -> u32;
        }
        "#.to_string());
        
        let args = create_test_args();
        
        // Run constructor check - should pass
        let constructor_check = ConstructorCheck::new();
        let constructor_result = constructor_check.run(&args, &mut context);
        assert!(constructor_result.is_ok());
        
        // Run upgrade function check - should fail
        let upgrade_check = UpgradeFunctionCheck::new();
        let upgrade_result = upgrade_check.run(&args, &mut context);
        assert!(upgrade_result.is_err());
    }
    
    #[test]
    fn test_both_checks_fail() {
        let mut context = SecurityCheckContext::new();
        // Define a contract interface WITH constructor and WITHOUT upgrade function
        context.contract_interface = Some(r#"
        #[soroban_sdk::contractargs(name = "Args")]
        #[soroban_sdk::contractclient(name = "Client")]
        pub trait Contract {
            fn __constructor(env: soroban_sdk::Env, admin: soroban_sdk::Address);
            fn handle_upgrade(env: soroban_sdk::Env);
            fn version(env: soroban_sdk::Env) -> u32;
        }
        "#.to_string());
        
        let args = create_test_args();
        
        // Run constructor check - should fail
        let constructor_check = ConstructorCheck::new();
        let constructor_result = constructor_check.run(&args, &mut context);
        assert!(constructor_result.is_err());
        
        // Run upgrade function check - should fail
        let upgrade_check = UpgradeFunctionCheck::new();
        let upgrade_result = upgrade_check.run(&args, &mut context);
        assert!(upgrade_result.is_err());
    }
} 