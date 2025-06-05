#[cfg(test)]
mod integration_tests {
    use crate::security_checks::{SecurityCheckContext, SecurityCheck};
    use crate::UpgradeArgs;
    use crate::security_checks::constructor_check::ConstructorCheck;
    use crate::security_checks::upgrade_function_check::UpgradeFunctionCheck;
    use crate::security_checks::version_check::VersionCheck;
    
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
            force: false,
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
        
        // Note: Version check would fail in tests because we don't have real contracts
        // So we don't test it in the integration test
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

    #[test]
    fn test_version_check_unit_tests() {
        // Test the version comparison logic separately
        let version_check = VersionCheck::new();
        
        // Test version comparison
        assert!(version_check.compare_versions("1.0.0", "2.0.0").unwrap());
        assert!(!version_check.compare_versions("2.0.0", "1.0.0").unwrap());
        assert!(!version_check.compare_versions("1.0.0", "1.0.0").unwrap());
        
        // Test version extraction
        let metadata = r#"[{"sc_meta_v0":{"key":"binver","val":"1.5.2"}},{"sc_meta_v0":{"key":"rsver","val":"1.85.0"}}]"#;
        let version = version_check.extract_binver(metadata).unwrap();
        assert_eq!(version, "1.5.2");
    }
} 