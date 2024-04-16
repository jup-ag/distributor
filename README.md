# Merkle-distributor

A program and toolsets for distributing tokens efficiently via uploading a [Merkle root](https://en.wikipedia.org/wiki/Merkle_tree).

## Sharding merkle tree

Thanks Jito for excellent [Merkle-distributor project](https://github.com/jito-foundation/distributor), and Jupiter who forked the project and added some extra steps to make it works for a large set of addresses. 

## CLI
Build:

```
cargo build -p jup-scripts
```

Refer for merkle tree deployment in `deploy_template.sh`

## API
We can host API in local server 
```
cd api
cargo build
../target/debug/kamino-airdrop-api --merkle-tree-path [PATH_TO_FOLDER_STORE_ALL_MERKLE_TREES] --rpc-url [RPC] --mint [TOKEN_MINT] --base [BASE_KEY] --program-id [PROGRAM_ID]
```
