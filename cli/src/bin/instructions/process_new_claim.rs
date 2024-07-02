use solana_sdk::compute_budget::ComputeBudgetInstruction;

use crate::*;

pub fn process_new_claim(args: &Args, claim_args: &ClaimArgs) {
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");
    let program_client = args.get_program_client();
    let claimant = keypair.pubkey();
    println!("Claiming tokens for user {}...", claimant);

    let merkle_tree = AirdropMerkleTree::new_from_file(&claim_args.merkle_tree_path)
        .expect("failed to load merkle tree from file");

    let (distributor, _bump) = get_merkle_distributor_pda(
        &args.program_id,
        &args.base,
        &args.mint,
        merkle_tree.airdrop_version,
    );

    let distributor_state: MerkleDistributor = program_client.account(distributor).unwrap();

    // Get user's node in claim
    let node = merkle_tree.get_node(&claimant);

    let (claim_status_pda, _bump) = get_claim_status_pda(&args.program_id, &claimant, &distributor);

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());

    let claimant_ata = get_associated_token_address(&claimant, &args.mint);

    let mut ixs = vec![];

    // check priority fee
    if let Some(priority_fee) = args.priority_fee {
        ixs.push(ComputeBudgetInstruction::set_compute_unit_price(
            priority_fee,
        ));
    }

    match client.get_account(&claimant_ata) {
        Ok(_) => {}
        Err(e) => {
            // TODO: directly pattern match on error kind
            if e.to_string().contains("AccountNotFound") {
                println!("PDA does not exist. creating.");
                ixs.push(create_associated_token_account(
                    &claimant,
                    &claimant,
                    &args.mint,
                    &token::ID,
                ));
            } else {
                panic!("Error fetching PDA: {e}")
            }
        }
    }

    let (escrow, _bump) = Pubkey::find_program_address(
        &[
            b"Escrow".as_ref(),
            distributor_state.locker.as_ref(),
            claimant.key().as_ref(),
        ],
        &locked_voter::ID,
    );
    ixs.push(Instruction {
        program_id: args.program_id,
        accounts: merkle_distributor::accounts::NewClaim {
            distributor,
            claim_status: claim_status_pda,
            from: get_associated_token_address(&distributor, &args.mint),
            to: claimant_ata,
            claimant,
            token_program: token::ID,
            system_program: solana_program::system_program::ID,
            voter_program: locked_voter::ID,
            locker: distributor_state.locker,
            escrow,
            escrow_tokens: get_associated_token_address(&escrow, &args.mint),
        }
        .to_account_metas(None),
        data: merkle_distributor::instruction::NewClaim {
            amount_unlocked: node.amount(),
            amount_locked: 0,
            proof: node.proof.expect("proof not found"),
        }
        .data(),
    });

    let blockhash = client.get_latest_blockhash().unwrap();
    let tx =
        Transaction::new_signed_with_payer(&ixs, Some(&claimant.key()), &[&keypair], blockhash);

    let signature = client
        .send_and_confirm_transaction_with_spinner(&tx)
        .unwrap();
    println!("successfully created new claim with signature {signature:#?}");
}
