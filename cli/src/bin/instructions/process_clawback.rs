use crate::*;
use anchor_client::solana_sdk::compute_budget::ComputeBudgetInstruction;
use anchor_lang::system_program;
use anyhow::Error;
use std::{thread, time::Duration};

pub fn process_clawback(args: &Args, clawback_args: &ClawbackArgs) {
    for i in (1..10).rev() {
        match clawback(args, clawback_args) {
            Ok(_) => {
                println!("Done clawback");
                return;
            }
            Err(_) => {
                println!("Failed to clawback, retrying, {} time remaining", i);
                thread::sleep(Duration::from_secs(5));
                if i == 1 {
                    clawback(args, clawback_args).expect("Failed to clawback");
                }
            }
        }
    }
}

fn clawback(args: &Args, clawback_args: &ClawbackArgs) -> Result<()> {
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());
    let program = args.get_program_client();

    let mut is_error = false;

    let from_version = clawback_args.from_version;
    let to_version = clawback_args.to_version;

    for version in from_version..=to_version {
        let (distributor, _bump) =
            get_merkle_distributor_pda(&args.program_id, &args.mint, version);
        if client
            .get_account_with_commitment(&distributor, CommitmentConfig::confirmed())?
            .value
            .is_none()
        {
            // distributor is not existed, exit
            break;
        }
        let distributor_state = program.account::<MerkleDistributor>(distributor)?;
        if distributor_state.clawed_back {
            println!("already clawback {}", version);
            continue;
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
            client.get_latest_blockhash()?,
        );

        match client.send_and_confirm_transaction_with_spinner(&tx) {
            Ok(signature) => {
                println!(
                    "Successfully clawback airdrop version {} ! signature: {signature:#?}",
                    version
                );
            }
            Err(_err) => {
                // println!("airdrop version {} {}", version, err);
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
