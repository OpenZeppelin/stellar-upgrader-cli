#!/bin/bash

# Example of how to use the Stellar Upgrader plugin

# Replace with your contract ID
CONTRACT_ID="CABY2EPFRLWMDTOQJMOSKM2LPZZ22LUKD5LE2MW35PY3T7FURARQDGMX"

# Replace with your WASM hash
WASM_HASH="9ab3011a533a116f82f99ebcd00e72cdca5e42159aaca379fd249fdbd982d9ff"

# Without the plugin (verbose way):
echo "Without the plugin:"
echo "stellar contract invoke \\"
echo "  --id $CONTRACT_ID \\"
echo "  --source alice \\"
echo "  --network testnet \\"
echo "  -- \\"
echo "  upgrade \\"
echo "  --new_wasm_hash $WASM_HASH"
echo ""

# With the plugin (simplified):
echo "With the plugin:"
echo "stellar contract upgrade --id $CONTRACT_ID --wasm-hash $WASM_HASH" 