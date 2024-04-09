use anchor_client::solana_sdk::compute_budget::ComputeBudgetInstruction;
use jito_merkle_tree::airdrop_merkle_tree::UserProof;

use crate::*;

pub fn process_claim_from_api(args: &Args, claim_args: &ClaimFromApiArgs) {
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");
    let claimant = keypair.pubkey();

    let kv_proof: UserProof = reqwest::blocking::get(format!(
        "{}/{}/{}",
        claim_args.root_api,
        args.mint.to_string(),
        claimant.to_string()
    ))
    .unwrap()
    .json()
    .unwrap();

    let distributor = Pubkey::from_str(&kv_proof.merkle_tree).unwrap();

    let (claim_status_pda, _bump) = get_claim_status_pda(&args.program_id, &claimant, &distributor);

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());

    let mut ixs = vec![];
    // check priority fee
    if let Some(priority_fee) = args.priority_fee {
        ixs.push(ComputeBudgetInstruction::set_compute_unit_price(
            priority_fee,
        ));
    }

    let claimant_ata = get_associated_token_address(&claimant, &args.mint);

    if client.get_account_data(&claimant_ata).is_err() {
        ixs.push(
            spl_associated_token_account::instruction::create_associated_token_account(
                &keypair.pubkey(),
                &claimant_ata,
                &args.mint,
                &spl_token::ID,
            ),
        );
    }

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
        }
        .to_account_metas(None),
        data: merkle_distributor::instruction::NewClaim {
            amount_unlocked: kv_proof.amount,
            amount_locked: 0,
            proof: kv_proof.proof,
        }
        .data(),
    });

    // check if destination_owner is not claimant
    if claim_args.destination_owner != claimant {
        let destination_ata = get_associated_token_address(&claimant, &args.mint);
        if client.get_account_data(&destination_ata).is_err() {
            ixs.push(
                spl_associated_token_account::instruction::create_associated_token_account(
                    &keypair.pubkey(),
                    &destination_ata,
                    &args.mint,
                    &spl_token::ID,
                ),
            );
        }
        ixs.push(
            spl_token::instruction::transfer(
                &spl_token::ID,
                &claimant_ata,
                &destination_ata,
                &claimant,
                &vec![],
                kv_proof.amount,
            )
            .unwrap(),
        );
    }

    let blockhash = client.get_latest_blockhash().unwrap();
    let tx =
        Transaction::new_signed_with_payer(&ixs, Some(&claimant.key()), &[&keypair], blockhash);

    let signature = client
        .send_and_confirm_transaction_with_spinner(&tx)
        .unwrap();
    println!("successfully claimed tokens with signature {signature:#?}",);
}
