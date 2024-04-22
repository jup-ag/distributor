use solana_sdk::compute_budget::ComputeBudgetInstruction;

use crate::*;
pub fn process_set_admin(args: &Args, set_admin_args: &SetAdminArgs) {
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());
    let program = args.get_program_client();

    let from_version = set_admin_args.from_version;
    let to_version = set_admin_args.from_version;
    for i in from_version..=to_version {
        let (distributor, _bump) =
            get_merkle_distributor_pda(&args.program_id, &args.base, &args.mint, i);

        loop {
            let distributor_state = program.account::<MerkleDistributor>(distributor).unwrap();
            if distributor_state.admin == set_admin_args.new_admin {
                println!("already the same skip airdrop version {}", i);
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
                accounts: merkle_distributor::accounts::SetAdmin {
                    distributor,
                    admin: keypair.pubkey(),
                    new_admin: set_admin_args.new_admin,
                }
                .to_account_metas(None),
                data: merkle_distributor::instruction::SetAdmin {}.data(),
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
                        "Successfully set admin {} airdrop version {} ! signature: {signature:#?}",
                        set_admin_args.new_admin, i
                    );
                    break;
                }
                Err(err) => {
                    println!("airdrop version {} {}", i, err);
                }
            }
        }
    }
}
