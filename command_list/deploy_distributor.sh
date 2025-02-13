csv_path="[path to csv]"
merkle_tree_path="[path to folder that stores merkle tree proof]"
token_decimals="[Token decimals]"
token_mint="[Token mint address]"

# can genrate a random keypair to sign
base_path="[Path to base key]"  

# Keypair_path to the address that will deploy distributor (admin), that address also need to prepare enough token to fund merkle tree
# Note: after deployment is suscessful, and team doesn't need to update anything, admin of distributor should be transfer  to team's multisig
# Command: 
# target/debug/cli --mint $token_mint --base $base_key --keypair-path $keypair_path --rpc-url $rpc set-admin --new-admin $new_admin --merkle-tree-path $merkle_tree_path
keypair_path="[Path to keypair]"

rpc="http://127.0.0.1:8899"
# clawback_start_ts should be in future, at least 1 day from current time
clawback_start_ts="[Clawback date]"

# refer command to get enable slot, user is able to claim after enable_slot
# target/debug/cli --rpc-url $rpc slot-by-time --timestamp $time_to_enable_claiming
# Note: because real timestamp may be diff from slot timestamp, so team may want to adjust enable slot in the time that closes to the launch time, so admin can adjust enable slot by the command
# target/debug/cli --mint $token_mint --base $base_key --priority-fee $priority_fee --keypair-path $keypair_path --rpc-url $rpc set-enable-slot --merkle-tree-path $merkle_tree_path --slot $enable_slot
activation_point="[Activation slot]"  
activation_type=0

# the address that will receive token that user haven't claimed yet, should be team's multisig
clawback_receiver_owner="[Clawback receiver owner]"  

bonus_vesting_duration="[Bonus vesting duration]"
bonus_multiplier="[Bonus multiplier]"

operator="[Operator]"
locker="[Locker]"
claim_type=[Claim type]

## caculated variable, can ignore this
# kv_path="[path to kv proofs]"
priority_fee=1000000 # priority fee, can use other number
max_nodes_per_tree=12000 # default value, can ignore the field
base_key=$(solana-keygen pubkey $base_path)
end_vesting_ts=$((clawback_start_ts - 86400)) # we dont care for end_vesting_ts and start_vesting ts
start_vesting_ts=$((end_vesting_ts - 1))
admin=$(solana-keygen pubkey $keypair_path)

echo "create merkle tree proof"
target/debug/cli create-merkle-tree --csv-path $csv_path --merkle-tree-path $merkle_tree_path --max-nodes-per-tree $max_nodes_per_tree --amount 0 --decimals $token_decimals

echo "deploy distributor"
target/debug/cli --mint $token_mint --priority-fee $priority_fee --keypair-path $keypair_path --rpc-url $rpc new-distributor --start-vesting-ts $start_vesting_ts --end-vesting-ts $end_vesting_ts --merkle-tree-path $merkle_tree_path --base-path $base_path --clawback-start-ts $clawback_start_ts --activation-point $activation_point --activation-type $activation_type --clawback-receiver-owner $clawback_receiver_owner --closable --bonus-vesting-duration $bonus_vesting_duration --bonus-multiplier $bonus_multiplier --operator $operator --locker $locker --claim-type $claim_type

echo "fund distributor"
target/debug/cli --mint $token_mint --priority-fee $priority_fee --base $base_key --keypair-path $keypair_path --rpc-url $rpc fund-all --merkle-tree-path $merkle_tree_path

echo "verify"
target/debug/cli --mint $token_mint --base $base_key --rpc-url $rpc verify --merkle-tree-path $merkle_tree_path --clawback-start-ts $clawback_start_ts --activation-point $activation_point --activation-type $activation_type --admin $admin --clawback-receiver-owner $clawback_receiver_owner --closable --bonus-vesting-duration $bonus_vesting_duration --bonus-multiplier $bonus_multiplier