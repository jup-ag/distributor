use crate::*;

pub fn process_clawback(args: &Args, clawback_args: &ClawbackArgs) {
    let payer_keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());

    let program = args.get_program_client();

    let (distributor, _bump) =
        get_merkle_distributor_pda(&args.program_id, &args.mint, clawback_args.airdrop_version);

    let distributor_state: MerkleDistributor = program.account(distributor).unwrap();

    let from = get_associated_token_address(&distributor, &args.mint);
    println!("from: {from}");

    let clawback_ix = Instruction {
        program_id: args.program_id,
        accounts: merkle_distributor::accounts::Clawback {
            distributor,
            from,
            to: distributor_state.clawback_receiver,
            claimant: payer_keypair.pubkey(),
            system_program: solana_program::system_program::ID,
            token_program: token::ID,
        }
        .to_account_metas(None),
        data: merkle_distributor::instruction::Clawback {}.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[clawback_ix],
        Some(&payer_keypair.pubkey()),
        &[&payer_keypair],
        client.get_latest_blockhash().unwrap(),
    );

    let signature = client
        .send_and_confirm_transaction_with_spinner(&tx)
        .unwrap();

    println!("Successfully clawed back funds! signature: {signature:#?}");
}
