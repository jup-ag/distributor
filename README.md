# Merkle-distributor

A program and toolsets for distributing tokens efficiently via uploading a [Merkle root](https://en.wikipedia.org/wiki/Merkle_tree).

## Sharding merkle tree

Thanks Jito for excellent [Merkle-distributor project](https://github.com/jito-foundation/distributor). In Jupiter, We fork the project and add some extra steps to make it works for a large set of addresses. 

There are issues if the number of airdrop addresses increases:
- The size of the proof increases, that may be over solana transaction size limit.
- Too many write locked accounts duration hot claming event, so only some transactions are get through. 

In order to tackle it, we break the large set of addresses to smaller merkle trees, like 12000 addresses for each merkle tree. Therefore, when user claim, that would write lock on different accounts as well as reduces proof size. 

Before are follow toolset to build sharding merkle trees

## CLI
Build:

```
cd cli
cargo build
```

Refer for merkle tree deployment in `deploy_template.sh`

## API
We can host API in local server 
```
cd api
cargo build
../target/debug/jupiter-airdrop-api --merkle-tree-path [PATH_TO_FOLDER_STORE_ALL_MERKLE_TREES] --rpc-url [RPC] --mint [TOKEN_MINT] --base [BASE_KEY] --program-id [PROGRAM_ID]
```
