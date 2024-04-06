root=$(pwd)


### CONFIG ZONE
csv_path="${root}/list/tensor.csv"
merkle_tree_path="${root}/proofs/mk/tensor"
kv_path="${root}/proofs/kv/tensor"
token_decimals=9
token_mint="ETdfSQc9naEwJyhtE5EQL2R7fcRXUsw1DuFiX5Hk97RX"
base_key="4w5sRAXNV6PKmp8bFNvo7XuTM8djZfzfxPfbHVfMcBW7"
### END CONFIG ZONE


cli=$root/target/debug/cli
max_nodes_per_tree=12000 # default value, can ignore the field

echo "create merkle tree proof"
$cli create-merkle-tree --csv-path $csv_path --merkle-tree-path $merkle_tree_path --max-nodes-per-tree $max_nodes_per_tree --amount 0 --decimals $token_decimals

echo "generate kv proofs"
$cli --mint $token_mint --base $base_key generate-kv-proof --merkle-tree-path $merkle_tree_path --kv-path $kv_path --max-entries-per-file 100000
