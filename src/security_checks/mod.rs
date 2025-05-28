mod constructor_check;
mod contract_info;
mod upgrade_function_check;
#[cfg(test)]
mod tests;

use crate::UpgradeArgs;

pub struct SecurityCheckContext {
    pub contract_interface: Option<String>,
}

impl SecurityCheckContext {
    pub fn new() -> Self {
        SecurityCheckContext {
            contract_interface: None,
        }
    }
}

pub trait SecurityCheck {
    fn name(&self) -> &str;
    fn run(&self, args: &UpgradeArgs, context: &mut SecurityCheckContext) -> Result<(), String>;
}

// Register all security checks here
pub fn get_security_checks() -> Vec<Box<dyn SecurityCheck>> {
    vec![
        Box::new(constructor_check::ConstructorCheck::new()),
        Box::new(upgrade_function_check::UpgradeFunctionCheck::new()),
    ]
}

// Run all security checks
pub fn run_all_checks(args: &UpgradeArgs) -> Result<(), String> {
    let mut context = SecurityCheckContext::new();
    
    // First, get contract info which will be used by multiple checks
    contract_info::fetch_contract_interface(args, &mut context)?;
    
    let checks = get_security_checks();
    
    for check in checks {
        println!("Running security check: {}", check.name());
        check.run(args, &mut context)?;
    }
    
    Ok(())
} 