# Stellar Contract Upgrader Plugin

A CLI plugin for Stellar that simplifies the process of upgrading smart contracts while performing important security checks.

## Overview

This plugin provides a streamlined interface for upgrading Stellar smart contracts, replacing the verbose standard command with a simpler syntax. Before executing the upgrade, it performs several security checks to ensure the upgrade will be successful and the contract will remain upgradeable after the operation.

## Installation

### Prerequisites

- Rust and Cargo installed
- Stellar CLI installed and configured

### Install from source

```bash
# Clone the repository
git clone https://github.com/OpenZeppelin/stellar-upgrader-cli.git
cd stellar-upgrader

# Build and install
cargo install --path .
```

## Usage

### Basic Usage

```bash
# Instead of the verbose command:
stellar contract invoke \
  --id CONTRACT_ID \
  --source alice \
  --network testnet \
  -- \
  upgrade \
  --new_wasm_hash 9ab3011a533a116f82f99ebcd00e72cdca5e42159aaca379fd249fdbd982d9ff

# Use the simplified command:
stellar contract upgrade --id CONTRACT_ID --wasm-hash 9ab3011a533a116f82f99ebcd00e72cdca5e42159aaca379fd249fdbd982d9ff

# You can also specify network and source (optional):
stellar contract upgrade --id CONTRACT_ID --wasm-hash HASH --network testnet --source alice

# Force upgrade and skip security checks (use with caution):
stellar contract upgrade --id CONTRACT_ID --wasm-hash HASH --force
```

### Advanced Options

The plugin supports all the parameters of the original `stellar contract invoke` command:

```bash
stellar contract upgrade \
  --id CONTRACT_ID \
  --wasm-hash HASH \
  --source alice \
  --network testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --rpc-header "Authorization: Bearer token" \
  --network-passphrase "Test SDF Network ; September 2015" \
  --fee 200 \
  --is-view \
  --instructions 100000 \
  --build-only \
  --send yes \
  --cost \
  --force
```

#### Parameter Reference

| Parameter | Description |
|-----------|-------------|
| `--id` | Contract ID to upgrade (required) |
| `--wasm-hash` | The new WASM hash for the upgrade (required) |
| `--source` | Source account that will submit the transaction (default: "alice") |
| `--network` | Network to use: testnet, futurenet, mainnet (default: "testnet") |
| `--rpc-url` | RPC server endpoint |
| `--rpc-header` | RPC Header(s) to include in requests to the RPC provider |
| `--network-passphrase` | Network passphrase to sign the transaction |
| `--fee` | Fee amount for transaction, in stroops (1 stroop = 0.0000001 XLM) |
| `--is-view` | View the result by simulating, without signing or submitting the transaction |
| `--instructions` | Number of instructions to simulate |
| `--build-only` | Build the transaction and only write the base64 XDR to stdout |
| `--send` | Whether to send the transaction: "yes", "no", "default" |
| `--cost` | Output the cost execution to stderr |
| `--force` | Force the upgrade and skip all security checks (use with caution) |

## Security Checks

This plugin performs these security checks before executing the upgrade:

1. **Constructor Check**: Verifies the contract doesn't have a `__constructor` function, which could cause issues during upgrades.
   - ✅ Pass: No `__constructor` function found
   - ❌ Fail: `__constructor` function found, risk of issues during upgrade

2. **Upgrade Function Check**: Ensures the contract exposes an `upgrade` function with the correct signature.
   - ✅ Pass: `upgrade` function with proper `new_wasm_hash: soroban_sdk::BytesN<32>` parameter found
   - ❌ Fail: Missing upgrade function or incorrect signature, which would prevent future upgrades

All security checks must pass for the upgrade command to execute.

### Bypassing Security Checks

Sometimes, even with security checks failing, you may want to force an upgrade. This can be done using the `--force` flag:

```bash
stellar contract upgrade --id CONTRACT_ID --wasm-hash HASH --force
```

**⚠️ Warning**: Using `--force` skips all security checks and may result in:
- Upgrade failures
- Loss of contract upgradeability
- Unexpected contract behavior

Only use `--force` when you understand the risks and have manually verified the upgrade is safe.

## Development

### Project Structure

```
stellar-upgrader/
├── src/
│   ├── main.rs            # CLI entry point
│   ├── lib.rs             # Core functionality
│   └── security_checks/   # Modular security checks
│       ├── mod.rs         # Main security check module
│       ├── constructor_check.rs
│       ├── upgrade_function_check.rs
│       └── contract_info.rs
├── examples/              # Usage examples
└── tests/                 # Integration tests
```

### Adding New Security Checks

1. Create a new file in `src/security_checks/` (e.g., `new_check.rs`)
2. Implement the `SecurityCheck` trait
3. Add your check to the list in `src/security_checks/mod.rs`

Example:

```rust
// In new_check.rs
use crate::UpgradeArgs;
use super::{SecurityCheck, SecurityCheckContext};

pub struct NewCheck;

impl NewCheck {
    pub fn new() -> Self {
        NewCheck
    }
}

impl SecurityCheck for NewCheck {
    fn name(&self) -> &str {
        "New Security Check"
    }
    
    fn description(&self) -> &str {
        "Description of what this check does"
    }
    
    fn run(&self, _args: &UpgradeArgs, context: &mut SecurityCheckContext) -> Result<(), String> {
        // Check implementation
        Ok(())
    }
}

// In mod.rs, add to get_security_checks()
pub fn get_security_checks() -> Vec<Box<dyn SecurityCheck>> {
    vec![
        Box::new(constructor_check::ConstructorCheck::new()),
        Box::new(upgrade_function_check::UpgradeFunctionCheck::new()),
        Box::new(new_check::NewCheck::new()),
    ]
}
```

### Running Tests

```bash
cargo test
```
