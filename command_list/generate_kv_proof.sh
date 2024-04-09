root=$(pwd)


### CONFIG ZONE
csv_path="${root}/list/tensor.csv"
merkle_tree_path="${root}/proofs/mk/tensor"
kv_path="${root}/proofs/kv/tensor"
token_decimals=9
token_mint="4fu4KCXhtKacafbPL7Z4TyBUXyvxfmBqhkf9MdcwMwsz"
base_key="22Uf9RMMT7MS1Wftthds36URvuq6k1bhNQRFufNrCJUp"
### END CONFIG ZONE


cli=$root/target/debug/cli
max_nodes_per_tree=12000 # default value, can ignore the field

echo "create merkle tree proof"
$cli create-merkle-tree --csv-path $csv_path --merkle-tree-path $merkle_tree_path --max-nodes-per-tree $max_nodes_per_tree --amount 0 --decimals $token_decimals

echo "generate kv proofs"
$cli --mint $token_mint --base $base_key generate-kv-proof --merkle-tree-path $merkle_tree_path --kv-path $kv_path --max-entries-per-file 100000
