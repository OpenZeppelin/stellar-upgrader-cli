# Stellar Contract Upgrader Plugin

A CLI plugin for Stellar that simplifies the process of upgrading smart contracts.

## Installation

```bash
cargo install stellar-upgrader
```

## Usage

Simplify the contract upgrade process:

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
```

This plugin performs several security checks before executing the upgrade:
- Security check 1
- Security check 2
- Security check 3

## License

This project is licensed under the Apache License, Version 2.0. 