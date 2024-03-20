csv_path="[path to csv]"
merkle_tree_path="[path to folder that stores merkle tree proof]"
max_nodes_per_tree=12000
token_decimals="[Token decimals]"
token_mint="[Token mint address]"
keypair_path="[Path to keypair]"
admin=$(solana-keygen pubkey $keypair_path)
rpc="http://127.0.0.1:8899"
clawback_start_ts="[Clawback date, should be in future]"
# we dont care for end_vesting_ts and start_vesting ts
end_vesting_ts=$((clawback_start_ts - 86400))
start_vesting_ts=$((end_vesting_ts - 1))
enable_slot="[Enable slot]"
base_path="[Path to base key]"
base_key=$(solana-keygen pubkey $base_path)
clawback_receiver_owner="[Clawback receiver owner]"


echo "create merkle tree proof"
target/debug/cli create-merkle-tree --csv-path $csv_path --merkle-tree-path $merkle_tree_path --max-nodes-per-tree $max_nodes_per_tree --amount 0 --decimals $token_decimals

echo "deploy distributor"
target/debug/cli --mint $token_mint --keypair-path $keypair_path --rpc-url $rpc new-distributor --start-vesting-ts $start_vesting_ts --end-vesting-ts $end_vesting_ts --merkle-tree-path $merkle_tree_path --base-path $base_path --clawback-start-ts $clawback_start_ts --enable-slot $enable_slot --clawback-receiver-owner $clawback_receiver_owner

echo "fund distributor"
target/debug/cli --mint $token_mint --base $base_key --keypair-path $keypair_path --rpc-url $rpc fund-all --merkle-tree-path $merkle_tree_path

echo "verify"
target/debug/cli --mint $token_mint --base $base_key --rpc-url $rpc verify --merkle-tree-path $merkle_tree_path --clawback-start-ts $clawback_start_ts --enable-slot  $enable_slot --admin $admin --clawback-receiver-owner $clawback_receiver_owner
