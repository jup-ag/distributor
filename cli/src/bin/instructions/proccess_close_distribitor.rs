use anchor_client::solana_sdk::compute_budget::ComputeBudgetInstruction;

use crate::*;

pub fn process_close_distributor(args: &Args, close_distributor_args: &CloseDistributorArgs) {
    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());
    let mut paths: Vec<_> = fs::read_dir(&close_distributor_args.merkle_tree_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    for file in paths {
        let single_tree_path = file.path();

        let merkle_tree =
            AirdropMerkleTree::new_from_file(&single_tree_path).expect("failed to read");

        if close_distributor_args.airdrop_version.is_some() {
            let airdrop_version = close_distributor_args.airdrop_version.unwrap();
            if airdrop_version != merkle_tree.airdrop_version {
                continue;
            }
        }

        let (distributor, _bump) = get_merkle_distributor_pda(
            &args.program_id,
            &args.base,
            &args.mint,
            merkle_tree.airdrop_version,
        );
        let program = args.get_program_client();
        let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
            .expect("Failed reading keypair file");
        // verify distributor is existed
        let merkle_distributor_state = program.account::<MerkleDistributor>(distributor);
        if merkle_distributor_state.is_err() {
            println!("skip version {}", merkle_tree.airdrop_version);
            continue;
        }

        let mut ixs = vec![];
        // check priority fee
        if let Some(priority_fee) = args.priority_fee {
            ixs.push(ComputeBudgetInstruction::set_compute_unit_price(
                priority_fee,
            ));
        }

        let merkle_distributor_state = merkle_distributor_state.unwrap();

        let destination_token_account = spl_associated_token_account::get_associated_token_address(
            &keypair.pubkey(),
            &args.mint,
        );

        if client.get_account_data(&destination_token_account).is_err() {
            ixs.push(
                spl_associated_token_account::instruction::create_associated_token_account(
                    &keypair.pubkey(),
                    &keypair.pubkey(),
                    &args.mint,
                    &spl_token::ID,
                ),
            );
        }

        ixs.push(Instruction {
            program_id: args.program_id,
            accounts: merkle_distributor::accounts::CloseDistributor {
                distributor,
                token_vault: merkle_distributor_state.token_vault,
                admin: keypair.pubkey(),
                destination_token_account,
                token_program: spl_token::ID,
            }
            .to_account_metas(None),
            data: merkle_distributor::instruction::CloseDistributor {}.data(),
        });

        let blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(
            &ixs,
            Some(&keypair.pubkey()),
            &[&keypair],
            blockhash,
        );
        match client.send_transaction(&tx) {
            Ok(_) => {
                println!(
                    "done close merkle distributor version {} {:?}",
                    merkle_tree.airdrop_version,
                    tx.get_signature(),
                );
            }
            Err(e) => {
                println!(
                    "Failed to close MerkleDistributor version {}: {:?}",
                    merkle_tree.airdrop_version, e
                );
            }
        }

        if close_distributor_args.airdrop_version.is_some() {
            let airdrop_version = close_distributor_args.airdrop_version.unwrap();
            if airdrop_version == merkle_tree.airdrop_version {
                break;
            }
        }
    }
}
