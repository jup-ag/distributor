use crate::*;

pub fn view_distributor(args: &Args, view_distributor_args: &ViewDistributorArgs) {
    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());

    let (distributor_pubkey, _bump) = get_merkle_distributor_pda(
        &args.program_id,
        &args.base,
        &args.mint,
        view_distributor_args.airdrop_version,
    );

    println!("distributor address: {}", distributor_pubkey);

    if let Some(account) = client
        .get_account_with_commitment(&distributor_pubkey, CommitmentConfig::confirmed())
        .unwrap()
        .value
    {
        println!("merkle distributor account exists");
        let distributor = MerkleDistributor::try_deserialize(&mut account.data.as_slice()).unwrap();
        println!("{:?}", distributor);
    } else {
        println!("merkle distributor account doesn't exist");
    }
}
