### CONFIG ZONE
token_mint="TNSRxcUxoT9xBG3de7PiJyTDYu7kskLqcpddxnEJAS6"
rpc="[RPC]"
destination_owner="[Destination owner, can be different from claimant]"
keypair_path="[Path to keypair]"
priority_fee=1000 # priority fee, can update this
### END CONFIG ZONE

root=$(pwd)
cli=$root/target/debug/cli

echo "Claim tokens"
$cli --mint $token_mint --priority-fee $priority_fee --rpc-url $rpc --keypair-path $keypair_path claim-from-api --destination-owner $destination_owner