use std::{thread, time::Duration};

use anchor_client::solana_sdk::compute_budget::ComputeBudgetInstruction;
use anyhow::{Error, Result};

use crate::*;

pub fn process_new_distributor(args: &Args, new_distributor_args: &NewDistributorArgs) {
    println!("creating new distributor with args: {new_distributor_args:#?}");

    for i in (1..10).rev() {
        match create_new_distributor(args, new_distributor_args, 0, 0) {
            Ok(_) => {
                println!("Done create all distributors");
                return;
            }
            Err(_) => {
                println!(
                    "Failed to create distributors, retrying, {} time remaining",
                    i
                );
                thread::sleep(Duration::from_secs(5));
                if i == 1 {
                    create_new_distributor(args, new_distributor_args, 0, 0)
                        .expect("Failed to create distributors");
                }
            }
        }
    }
}

pub fn process_new_distributor_with_bonus(
    args: &Args,
    new_distributor_args: &NewDistributorWithBonusArgs,
) {
    println!("creating new distributor with args: {new_distributor_args:#?}");

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());
    let average_slot_time = get_average_slot_time(&client).unwrap();
    let bonus_vesting_slot_duration = new_distributor_args
        .bonus_vesting_duration
        .checked_mul(1000)
        .unwrap()
        .checked_div(average_slot_time)
        .unwrap();

    for i in (1..10).rev() {
        match create_new_distributor(
            args,
            &new_distributor_args.to_new_distributor_args(),
            new_distributor_args.bonus_multiplier,
            bonus_vesting_slot_duration,
        ) {
            Ok(_) => {
                println!("Done create all distributors");
                return;
            }
            Err(_) => {
                println!(
                    "Failed to create distributors, retrying, {} time remaining",
                    i
                );
                thread::sleep(Duration::from_secs(5));
                if i == 1 {
                    create_new_distributor(
                        args,
                        &new_distributor_args.to_new_distributor_args(),
                        new_distributor_args.bonus_multiplier,
                        bonus_vesting_slot_duration,
                    )
                    .expect("Failed to create distributors");
                }
            }
        }
    }
}

fn create_new_distributor(
    args: &Args,
    new_distributor_args: &NewDistributorArgs,
    bonus_multiplier: u64,
    bonus_vesting_slot_duration: u64,
) -> Result<()> {
    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap()).unwrap();
    let base = read_keypair_file(&new_distributor_args.base_path).unwrap();
    let mut paths: Vec<_> = fs::read_dir(&new_distributor_args.merkle_tree_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    let mut is_error = false;
    for file in paths {
        let single_tree_path = file.path();

        let merkle_tree = AirdropMerkleTree::new_from_file(&single_tree_path).unwrap();

        let total_bonus = merkle_tree
            .max_total_claim
            .checked_mul(bonus_multiplier)
            .unwrap();

        if new_distributor_args.airdrop_version.is_some() {
            let airdrop_version = new_distributor_args.airdrop_version.unwrap();
            if airdrop_version != merkle_tree.airdrop_version {
                continue;
            }
        }
        let (distributor_pubkey, _bump) = get_merkle_distributor_pda(
            &args.program_id,
            &base.pubkey(),
            &args.mint,
            merkle_tree.airdrop_version,
        );

        if let Some(account) = client
            .get_account_with_commitment(&distributor_pubkey, CommitmentConfig::confirmed())?
            .value
        {
            println!(
                "merkle distributor {} account exists, checking parameters...",
                merkle_tree.airdrop_version
            );
            check_distributor_onchain_matches(
                &account,
                &merkle_tree,
                new_distributor_args,
                total_bonus,
                bonus_vesting_slot_duration,
                keypair.pubkey(),
                base.pubkey(),
                &args,
            ).expect("merkle root on-chain does not match provided arguments! Confirm admin and clawback parameters to avoid loss of funds!");
            continue;
        }

        let mut ixs = vec![];

        // check priority fee
        if let Some(priority_fee) = args.priority_fee {
            ixs.push(ComputeBudgetInstruction::set_compute_unit_price(
                priority_fee,
            ));
        }

        let token_vault = spl_associated_token_account::get_associated_token_address(
            &distributor_pubkey,
            &args.mint,
        );
        if client.get_account_data(&token_vault).is_err() {
            ixs.push(
                spl_associated_token_account::instruction::create_associated_token_account(
                    &keypair.pubkey(),
                    &distributor_pubkey,
                    &args.mint,
                    &spl_token::ID,
                ),
            );
        }
        let clawback_receiver = spl_associated_token_account::get_associated_token_address(
            &new_distributor_args.clawback_receiver_owner,
            &args.mint,
        );

        if client.get_account_data(&clawback_receiver).is_err() {
            ixs.push(
                spl_associated_token_account::instruction::create_associated_token_account(
                    &keypair.pubkey(),
                    &new_distributor_args.clawback_receiver_owner,
                    &args.mint,
                    &spl_token::ID,
                ),
            );
        }

        if total_bonus == 0 {
            ixs.push(Instruction {
                program_id: args.program_id,
                accounts: merkle_distributor::accounts::NewDistributor {
                    base: base.pubkey(),
                    clawback_receiver,
                    mint: args.mint,
                    token_vault,
                    distributor: distributor_pubkey,
                    system_program: solana_program::system_program::id(),
                    associated_token_program: spl_associated_token_account::ID,
                    token_program: token::ID,
                    admin: keypair.pubkey(),
                }
                .to_account_metas(None),
                data: merkle_distributor::instruction::NewDistributor {
                    version: merkle_tree.airdrop_version,
                    root: merkle_tree.merkle_root,
                    max_total_claim: merkle_tree.max_total_claim,
                    max_num_nodes: merkle_tree.max_num_nodes,
                    start_vesting_ts: new_distributor_args.start_vesting_ts,
                    end_vesting_ts: new_distributor_args.end_vesting_ts,
                    clawback_start_ts: new_distributor_args.clawback_start_ts,
                    enable_slot: new_distributor_args.enable_slot,
                    closable: new_distributor_args.closable,
                }
                .data(),
            });
        } else {
            ixs.push(Instruction {
                program_id: args.program_id,
                accounts: merkle_distributor::accounts::NewDistributor {
                    base: base.pubkey(),
                    clawback_receiver,
                    mint: args.mint,
                    token_vault,
                    distributor: distributor_pubkey,
                    system_program: solana_program::system_program::id(),
                    associated_token_program: spl_associated_token_account::ID,
                    token_program: token::ID,
                    admin: keypair.pubkey(),
                }
                .to_account_metas(None),
                data: merkle_distributor::instruction::NewDistributor2 {
                    version: merkle_tree.airdrop_version,
                    root: merkle_tree.merkle_root,
                    total_claim: merkle_tree.max_total_claim,
                    max_num_nodes: merkle_tree.max_num_nodes,
                    start_vesting_ts: new_distributor_args.start_vesting_ts,
                    end_vesting_ts: new_distributor_args.end_vesting_ts,
                    clawback_start_ts: new_distributor_args.clawback_start_ts,
                    enable_slot: new_distributor_args.enable_slot,
                    closable: new_distributor_args.closable,
                    total_bonus,
                    bonus_vesting_slot_duration,
                }
                .data(),
            });
        }

        let blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(
            &ixs,
            Some(&keypair.pubkey()),
            &[&keypair, &base],
            blockhash,
        );

        // See comments on new_distributor instruction inside the program to ensure this transaction
        // didn't get frontrun.
        // If this fails, make sure to run it again.

        if new_distributor_args.skip_verify {
            match client.send_transaction(&tx) {
                Ok(_) => {
                    println!(
                        "done create merkle distributor version {} {:?}",
                        merkle_tree.airdrop_version,
                        tx.get_signature(),
                    );
                }
                Err(e) => {
                    is_error = true;
                    println!("Failed to create MerkleDistributor: {:?}", e);
                }
            }
        } else {
            match client.send_and_confirm_transaction_with_spinner(&tx) {
                Ok(_) => {
                    println!(
                        "done create merkle distributor version {} {:?}",
                        merkle_tree.airdrop_version,
                        tx.get_signature(),
                    );
                }
                Err(e) => {
                    is_error = true;
                    println!("Failed to create MerkleDistributor: {:?}", e);
                }
            }

            // double check someone didn't frontrun this transaction with a malicious merkle root
            if let Some(account) = client
                .get_account_with_commitment(&distributor_pubkey, CommitmentConfig::processed())?
                .value
            {
                check_distributor_onchain_matches(
                  &account,
                  &merkle_tree,
                  new_distributor_args,
                  total_bonus,
                  bonus_vesting_slot_duration,
                  keypair.pubkey(),
                  base.pubkey(),
                  args,
              ).expect("merkle root on-chain does not match provided arguments! Confirm admin and clawback parameters to avoid loss of funds!");
            }
        }

        if new_distributor_args.airdrop_version.is_some() {
            let airdrop_version = new_distributor_args.airdrop_version.unwrap();
            if airdrop_version == merkle_tree.airdrop_version {
                break;
            }
        }
    }
    if is_error {
        return Err(Error::msg(
            "There are some error when create merkle tree, retrying",
        ));
    }
    Ok(())
}
