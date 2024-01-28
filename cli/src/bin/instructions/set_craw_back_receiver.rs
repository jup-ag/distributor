use crate::*;

pub fn set_craw_back_receiver(args: &Args, set_crawl_back_receiver: &CrawlBackReceiverArgs) {
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());

    let (distributor, _bump) = get_merkle_distributor_pda(
        &args.program_id,
        &args.mint,
        set_crawl_back_receiver.airdrop_version,
    );

    let new_clawback_account = spl_associated_token_account::get_associated_token_address(
        &set_crawl_back_receiver.receiver,
        &args.mint,
    );

    let set_crawlback_receiver_ix = Instruction {
        program_id: args.program_id,
        accounts: merkle_distributor::accounts::SetClawbackReceiver {
            distributor,
            admin: keypair.pubkey(),
            new_clawback_account,
        }
        .to_account_metas(None),
        data: merkle_distributor::instruction::SetClawbackReceiver {}.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[set_crawlback_receiver_ix],
        Some(&keypair.pubkey()),
        &[&keypair],
        client.get_latest_blockhash().unwrap(),
    );

    let signature = client
        .send_and_confirm_transaction_with_spinner(&tx)
        .unwrap();

    println!("Successfully set crawlback receiver! signature: {signature:#?}");
}
