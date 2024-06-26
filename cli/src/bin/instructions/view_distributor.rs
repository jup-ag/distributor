use crate::*;

pub fn view_distributors(args: &Args, view_distributor_args: &ViewDistributorsArgs) {
    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());

    let from_version = view_distributor_args.from_version;
    let to_version = view_distributor_args.to_version;
    for i in from_version..=to_version {
        let (distributor_pubkey, _bump) =
            get_merkle_distributor_pda(&args.program_id, &args.base, &args.mint, i);

        if let Some(account) = client
            .get_account_with_commitment(&distributor_pubkey, CommitmentConfig::confirmed())
            .unwrap()
            .value
        {
            let distributor =
                MerkleDistributor::try_deserialize(&mut account.data.as_slice()).unwrap();
            println!("pk {} version {} {:?}", distributor_pubkey, i, distributor);
        } else {
            println!("merkle distributor {} doesn't exist", i);
        }
    }
}

pub fn view_distributor_by_pubkey(args: &Args, distributor_pubkey: &Pubkey) {
    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());

    if let Some(account) = client
        .get_account_with_commitment(&distributor_pubkey, CommitmentConfig::confirmed())
        .unwrap()
        .value
    {
        let distributor = MerkleDistributor::try_deserialize(&mut account.data.as_slice()).unwrap();
        println!("{:?}", distributor);
    } else {
        println!("merkle distributor {} doesn't exist", distributor_pubkey);
    }
}
