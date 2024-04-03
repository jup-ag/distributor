use anchor_client::solana_sdk::compute_budget::ComputeBudgetInstruction;
use anchor_lang::system_program;

use crate::*;

pub fn process_clawback(args: &Args, clawback_args: &ClawbackArgs) {
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());
    let program = args.get_program_client();

    let mut paths: Vec<_> = fs::read_dir(&clawback_args.merkle_tree_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    for file in paths {
        let single_tree_path = file.path();

        let merkle_tree =
            AirdropMerkleTree::new_from_file(&single_tree_path).expect("failed to read");

        let (distributor, _bump) = get_merkle_distributor_pda(
            &args.program_id,
            &args.base,
            &args.mint,
            merkle_tree.airdrop_version,
        );

        loop {
            let distributor_state = program.account::<MerkleDistributor>(distributor).unwrap();
            if distributor_state.clawed_back {
                println!("already clawback {}", merkle_tree.airdrop_version);
                break;
            }

            let mut ixs = vec![];
            // check priority fee
            if let Some(priority_fee) = args.priority_fee {
                ixs.push(ComputeBudgetInstruction::set_compute_unit_price(
                    priority_fee,
                ));
            }

            ixs.push(Instruction {
                program_id: args.program_id,
                accounts: merkle_distributor::accounts::Clawback {
                    distributor,
                    from: distributor_state.token_vault,
                    token_program: spl_token::ID,
                    to: distributor_state.clawback_receiver,
                    claimant: keypair.pubkey(),
                    system_program: system_program::ID,
                }
                .to_account_metas(None),
                data: merkle_distributor::instruction::Clawback {}.data(),
            });

            let tx = Transaction::new_signed_with_payer(
                &ixs,
                Some(&keypair.pubkey()),
                &[&keypair],
                client.get_latest_blockhash().unwrap(),
            );

            match client.send_transaction(&tx) {
                Ok(signature) => {
                    println!(
                        "Successfully clawback airdrop version {} ! signature: {signature:#?}",
                        merkle_tree.airdrop_version
                    );
                    break;
                }
                Err(err) => {
                    println!("airdrop version {} {}", merkle_tree.airdrop_version, err);
                }
            }
        }
    }
}
