use solana_sdk::compute_budget::ComputeBudgetInstruction;

use crate::*;

pub fn process_set_enable_slot(args: &Args, set_enable_slot_args: &SetEnableSlotArgs) {
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());
    let program = args.get_program_client();

    let from_version = set_enable_slot_args.from_version;
    let to_version = set_enable_slot_args.to_version;
    for version in from_version..=to_version {
        let (distributor, _bump) =
            get_merkle_distributor_pda(&args.program_id, &args.base, &args.mint, version);

        loop {
            let distributor_state = program.account::<MerkleDistributor>(distributor).unwrap();
            if distributor_state.enable_slot == set_enable_slot_args.slot {
                println!("already set slot skip airdrop version {}", version);
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
                accounts: merkle_distributor::accounts::SetEnableSlot {
                    distributor,
                    admin: keypair.pubkey(),
                }
                .to_account_metas(None),
                data: merkle_distributor::instruction::SetEnableSlot {
                    enable_slot: set_enable_slot_args.slot,
                }
                .data(),
            });

            let tx = Transaction::new_signed_with_payer(
                &ixs,
                Some(&keypair.pubkey()),
                &[&keypair],
                client.get_latest_blockhash().unwrap(),
            );

            match client.send_and_confirm_transaction_with_spinner(&tx) {
                Ok(signature) => {
                    println!(
                        "Successfully set enable slot {} airdrop version {} ! signature: {signature:#?}",
                        set_enable_slot_args.slot, version
                    );
                    break;
                }
                Err(err) => {
                    println!("airdrop version {} {}", version, err);
                }
            }
        }
    }
}
