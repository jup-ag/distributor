use std::{thread, time::Duration};

use anchor_client::solana_sdk::compute_budget::ComputeBudgetInstruction;
use anyhow::Error;

use crate::*;
pub fn process_fund_all(args: &Args, fund_all_args: &FundAllArgs) {
    for i in (1..5).rev() {
        match fund_all(args, fund_all_args) {
            Ok(_) => {
                println!("Done fund all distributors");
                return;
            }
            Err(_) => {
                println!(
                    "Failed to fund distributors, retrying, {} time remaining",
                    i
                );
                thread::sleep(Duration::from_secs(5));
                if i == 1 {
                    fund_all(args, fund_all_args).expect("Failed to fund distributors");
                }
            }
        }
    }
}

fn fund_all(args: &Args, fund_all_args: &FundAllArgs) -> Result<()> {
    let program = args.get_program_client();
    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap()).unwrap();
    let mut paths: Vec<_> = fs::read_dir(&fund_all_args.merkle_tree_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    let source_vault = get_associated_token_address(&keypair.pubkey(), &args.mint);

    // println!("source vault {}", source_vault);

    let mut is_error = false;
    for file in paths {
        let single_tree_path = file.path();

        let merkle_tree = AirdropMerkleTree::new_from_file(&single_tree_path).unwrap();
        let (distributor_pubkey, _bump) = get_merkle_distributor_pda(
            &args.program_id,
            &args.base,
            &args.mint,
            merkle_tree.airdrop_version,
        );

        let token_vault = get_associated_token_address(&distributor_pubkey, &args.mint);

        let token_vault_state: TokenAccount = program.account(token_vault).unwrap();
        if token_vault_state.amount >= merkle_tree.max_total_claim {
            println!(
                "already fund airdrop version {}!",
                merkle_tree.airdrop_version
            );
            continue;
        }

        let mut ixs = vec![];
        // check priority fee
        if let Some(priority_fee) = args.priority_fee {
            ixs.push(ComputeBudgetInstruction::set_compute_unit_price(
                priority_fee,
            ));
        }
        ixs.push(
            spl_token::instruction::transfer(
                &spl_token::id(),
                &source_vault,
                &token_vault,
                &keypair.pubkey(),
                &[],
                merkle_tree.max_total_claim,
            )
            .unwrap(),
        );

        let tx = Transaction::new_signed_with_payer(
            &ixs,
            Some(&keypair.pubkey()),
            &[&keypair],
            client.get_latest_blockhash().unwrap(),
        );

        match client.send_and_confirm_transaction_with_spinner(&tx) {
            Ok(_) => {
                println!(
                    "done fund distributor version {} {:?}",
                    merkle_tree.airdrop_version,
                    tx.get_signature(),
                );
            }
            Err(e) => {
                println!(
                    "Failed to fund distributor version {}: {:?}",
                    merkle_tree.airdrop_version, e
                );
                is_error = true;
            }
        }
    }
    if is_error {
        return Err(Error::msg(
            "There are some error when fund merkle tree, retrying",
        ));
    }
    Ok(())
}
