csv_path="[path to csv]"
merkle_tree_path="[path to folder that stores merkle tree proof]"
kv_path="[path to kv proofs]"
max_nodes_per_tree=12000 # default value, can ignore the field
token_decimals="[Token decimals]"
token_mint="[Token mint address]"
keypair_path="[Path to keypair]"
rpc="http://127.0.0.1:8899"
clawback_start_ts="[Clawback date, should be in future]"
# we dont care for end_vesting_ts and start_vesting ts
enable_slot="[Enable slot]"
base_path="[Path to base key]"
clawback_receiver_owner="[Clawback receiver owner]"
priority_fee=3000000 # priority fee, can use other number


## caculated variable
base_key=$(solana-keygen pubkey $base_path)
end_vesting_ts=$((clawback_start_ts - 86400))
start_vesting_ts=$((end_vesting_ts - 1))
admin=$(solana-keygen pubkey $keypair_path)

echo "create merkle tree proof"
target/debug/cli create-merkle-tree --csv-path $csv_path --merkle-tree-path $merkle_tree_path --max-nodes-per-tree $max_nodes_per_tree --amount 0 --decimals $token_decimals

echo "generate kv proof"
target/debug/cli --mint $token_mint generate-kv-proof --merkle-tree-path $merkle_tree_path --kv-path $kv_path --max-entries-per-file 100000

echo "deploy distributor"
target/debug/cli --mint $token_mint --priority-fee $priority_fee --keypair-path $keypair_path --rpc-url $rpc new-distributor --start-vesting-ts $start_vesting_ts --end-vesting-ts $end_vesting_ts --merkle-tree-path $merkle_tree_path --base-path $base_path --clawback-start-ts $clawback_start_ts --enable-slot $enable_slot --clawback-receiver-owner $clawback_receiver_owner --closable

# echo "transfer admin to multisig"
echo "fund distributor"
target/debug/cli --mint $token_mint --priority-fee $priority_fee --base $base_key --keypair-path $keypair_path --rpc-url $rpc fund-all --merkle-tree-path $merkle_tree_path

# adjust enable slot
echo "verify"
target/debug/cli --mint $token_mint --base $base_key --rpc-url $rpc verify --merkle-tree-path $merkle_tree_path --clawback-start-ts $clawback_start_ts --enable-slot  $enable_slot --admin $admin --clawback-receiver-owner $clawback_receiver_owner --closable
