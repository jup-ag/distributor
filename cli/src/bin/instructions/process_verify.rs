use crate::*;

pub fn process_verify(args: &Args, verfify_args: &VerifyArgs) {
    let mut paths: Vec<_> = fs::read_dir(&verfify_args.merkle_tree_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    // TODO fix code
    let program = args.get_program_client();

    for file in paths {
        let single_tree_path = file.path();

        let merkle_tree =
            AirdropMerkleTree::new_from_file(&single_tree_path).expect("failed to read");

        if let Some(airdrop_version) = verfify_args.airdrop_version {
            if merkle_tree.airdrop_version != airdrop_version {
                continue;
            }
        }

        let (distributor_pubkey, _bump) = get_merkle_distributor_pda(
            &args.program_id,
            &args.base,
            &args.mint,
            merkle_tree.airdrop_version,
        );

        let total_bonus = verfify_args
            .bonus_multiplier
            .checked_mul(merkle_tree.total_unlocked_amount)
            .unwrap();

        println!(
            "Verify merkle tree airdrop version {} {}",
            merkle_tree.airdrop_version, distributor_pubkey
        );

        if !verfify_args.skip_verify_amount {
            let token_vault = get_associated_token_address(&distributor_pubkey, &args.mint);
            let token_vault_account: TokenAccount = program.account(token_vault).unwrap();
            assert_eq!(
                token_vault_account.amount,
                merkle_tree
                    .get_max_total_claim()
                    .checked_add(total_bonus)
                    .unwrap()
            );
        }

        let merke_tree_state: MerkleDistributor = program.account(distributor_pubkey).unwrap();
        assert_eq!(merke_tree_state.root, merkle_tree.merkle_root);

        assert_eq!(
            merke_tree_state.clawback_start_ts,
            verfify_args.clawback_start_ts
        );

        assert_eq!(merke_tree_state.closable, verfify_args.closable);

        assert_eq!(merke_tree_state.admin, verfify_args.admin);
        assert_eq!(merke_tree_state.enable_slot, verfify_args.enable_slot);

        // assert_eq!(
        //     merke_tree_state.airdrop_bonus.vesting_slot_duration,
        //     verfify_args.bonus_vesting_duration
        // );

        let clawback_receiver =
            get_associated_token_address(&verfify_args.clawback_receiver_owner, &args.mint);
        assert_eq!(merke_tree_state.clawback_receiver, clawback_receiver);
    }
}
