use anchor_lang::{
    solana_program::{program_option::COption, program_pack::Pack},
    AccountDeserialize, AccountSerialize, InstructionData, ToAccountMetas,
};
use litesvm::LiteSVM;
use solana_sdk::{
    account::Account as SdkAccount,
    clock::Clock,
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
mod common;
use common::PROGRAM_BYTES;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token::{
    id as spl_token_program_id,
    state::{Account as SplAccount, AccountState as SplAccountState, Mint as SplMint},
};

use merkle_distributor::{
    accounts as md_accounts,
    instruction as md_ix,
    state::{claim_status::ClaimStatus, merkle_distributor::MerkleDistributor},
    id as PROGRAM_ID,
};

fn pack_mint(mint_authority: &Pubkey, decimals: u8, supply: u64) -> Vec<u8> {
    let mut mint = SplMint::default();
    mint.is_initialized = true;
    mint.mint_authority = COption::from(Some(*mint_authority));
    mint.freeze_authority = None.into();
    mint.decimals = decimals;
    mint.supply = supply;
    let mut data = vec![0u8; SplMint::LEN];
    SplMint::pack(mint, &mut data).expect("pack mint");
    data
}

fn pack_token_account(mint: &Pubkey, owner: &Pubkey, amount: u64) -> Vec<u8> {
    let mut ta = SplAccount::default();
    ta.mint = *mint;
    ta.owner = *owner;
    ta.amount = amount;
    ta.state = SplAccountState::Initialized;
    ta.delegated_amount = 0;
    ta.close_authority = None.into();
    let mut data = vec![0u8; SplAccount::LEN];
    SplAccount::pack(ta, &mut data).expect("pack token account");
    data
}

fn svm_set_owned_with_data(
    svm: &mut LiteSVM,
    address: &Pubkey,
    owner: &Pubkey,
    data: Vec<u8>,
) {
    let min_lamports = svm.minimum_balance_for_rent_exemption(data.len());
    let acc = SdkAccount {
        lamports: min_lamports,
        data,
        owner: *owner,
        executable: false,
        rent_epoch: 0,
    };
    svm.set_account(*address, acc).expect("set_account");
}

fn send_tx(
    svm: &mut LiteSVM,
    payer: &Keypair,
    signers: &[&Keypair],
    ixs: &[Instruction],
) -> litesvm::types::TransactionMetadata {
    let bh: Hash = svm.latest_blockhash();
    let msg = Message::new(ixs, Some(&payer.pubkey()));
    let tx = Transaction::new(signers, msg, bh);
    svm.send_transaction(tx).expect("send tx")
}

fn bytes_changed(before: &[u8], after: &[u8]) -> usize {
    let min_len = before.len().min(after.len());
    let mut diff = 0;
    for i in 0..min_len {
        if before[i] != after[i] {
            diff += 1;
        }
    }
    diff + before.len().saturating_sub(min_len) + after.len().saturating_sub(min_len)
}

fn run_claim_locked_flow(use_fast_path: bool) {
    let mut svm = LiteSVM::new();
    let program_id = PROGRAM_ID();
    svm.add_program(program_id, PROGRAM_BYTES);

    let admin = Keypair::new();
    let base = Keypair::new();
    let claimant = Keypair::new();

    svm.airdrop(&admin.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&claimant.pubkey(), 2_000_000_000).unwrap();

    let mint = Pubkey::new_unique();
    let mint_data = pack_mint(&admin.pubkey(), 6, 0);
    svm_set_owned_with_data(&mut svm, &mint, &spl_token_program_id(), mint_data);

    let version: u64 = 77;
    let (distributor_pda, _bump) = Pubkey::find_program_address(
        &[
            b"MerkleDistributor",
            base.pubkey().as_ref(),
            mint.as_ref(),
            &version.to_le_bytes(),
        ],
        &program_id,
    );

    let token_vault = get_associated_token_address_with_program_id(
        &distributor_pda,
        &mint,
        &spl_token_program_id(),
    );
    let vault_initial_amount: u64 = 1_000_000_000_000;
    let vault_data = pack_token_account(&mint, &distributor_pda, vault_initial_amount);
    svm_set_owned_with_data(&mut svm, &token_vault, &spl_token_program_id(), vault_data);

    let clawback_receiver = Pubkey::new_unique();
    let clawback_data = pack_token_account(&mint, &admin.pubkey(), 0);
    svm_set_owned_with_data(&mut svm, &clawback_receiver, &spl_token_program_id(), clawback_data);

    let claimant_ata = get_associated_token_address_with_program_id(
        &claimant.pubkey(),
        &mint,
        &spl_token_program_id(),
    );
    let claimant_ata_data = pack_token_account(&mint, &claimant.pubkey(), 0);
    svm_set_owned_with_data(
        &mut svm,
        &claimant_ata,
        &spl_token_program_id(),
        claimant_ata_data,
    );

    let mut clock = svm.get_sysvar::<Clock>();
    let start_vesting_ts: i64 = clock.unix_timestamp + 1_000;
    let end_vesting_ts: i64 = start_vesting_ts + 1_000;
    let clawback_start_ts: i64 = end_vesting_ts + 24 * 3600 + 10;

    let amount_locked: u64 = 123_456;

    let root: [u8; 32] = [7u8; 32];

    let accounts_new = md_accounts::NewDistributor {
        distributor: distributor_pda,
        base: base.pubkey(),
        clawback_receiver,
        mint,
        token_vault,
        admin: admin.pubkey(),
        system_program: system_program::ID,
        associated_token_program: spl_associated_token_account::ID,
        token_program: spl_token_program_id(),
    };

    let ix_new = Instruction {
        program_id,
        accounts: accounts_new.to_account_metas(None),
        data: md_ix::NewDistributor {
            version,
            root,
            max_total_claim: 5_000_000,
            max_num_nodes: 10,
            start_vesting_ts,
            end_vesting_ts,
            clawback_start_ts,
            activation_point: 0,
            activation_type: 0,
            closable: true,
        }
        .data(),
    };
    let init_meta = send_tx(&mut svm, &admin, &[&admin, &base], &[ix_new]);
    assert!(
        init_meta
            .logs
            .iter()
            .any(|l| l.contains("New distributor")),
        "expected init log"
    );

    let (claim_status_pda, _cs_bump) = Pubkey::find_program_address(
        &[
            b"ClaimStatus",
            claimant.pubkey().as_ref(),
            distributor_pda.as_ref(),
        ],
        &program_id,
    );

    // Manually seed the claim_status account to simulate a completed `new_claim`.
    {
        let mut claim_status = ClaimStatus::default();
        claim_status.claimant = claimant.pubkey();
        claim_status.locked_amount = amount_locked;
        claim_status.locked_amount_withdrawn = 0;
        claim_status.unlocked_amount = 0;
        claim_status.closable = true;
        claim_status.admin = admin.pubkey();
        let mut data = vec![0u8; ClaimStatus::LEN];
        {
            let mut slice: &mut [u8] = data.as_mut_slice();
            claim_status
                .try_serialize(&mut slice)
                .expect("serialize claim_status");
        }
        let min_lamports = svm.minimum_balance_for_rent_exemption(data.len());
        let cs_acc = SdkAccount {
            lamports: min_lamports,
            data,
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        };
        svm.set_account(claim_status_pda, cs_acc)
            .expect("set claim_status");
    }

    // Fast forward so everything is unlocked.
    clock.unix_timestamp = end_vesting_ts + 10;
    svm.set_sysvar::<Clock>(&clock);

    let ix_data = if use_fast_path {
        md_ix::ClaimLocked {}.data()
    } else {
        md_ix::AnchorClaimLocked {}.data()
    };
    let claim_locked_accounts = vec![
        AccountMeta::new(distributor_pda, false),
        AccountMeta::new(claim_status_pda, false),
        AccountMeta::new(token_vault, false),
        AccountMeta::new(claimant_ata, false),
        AccountMeta::new(claimant.pubkey(), true),
        AccountMeta::new_readonly(spl_token_program_id(), false),
    ];
    let ix_claim_locked = Instruction {
        program_id,
        accounts: claim_locked_accounts,
        data: ix_data,
    };

    let payer_before = svm.get_account(&claimant.pubkey()).unwrap().lamports;
    let distributor_before = svm.get_account(&distributor_pda).unwrap().data.clone();
    let claim_status_before = svm.get_account(&claim_status_pda).unwrap().data.clone();
    let vault_before = svm.get_account(&token_vault).unwrap().data.clone();
    let to_before = svm.get_account(&claimant_ata).unwrap().data.clone();

    let meta = send_tx(&mut svm, &claimant, &[&claimant], &[ix_claim_locked]);
    assert!(meta.compute_units_consumed > 0);
    let logs = meta.logs.join("\n");
    assert!(
        logs.contains("Withdrew amount"),
        "expected withdraw log, got {logs}"
    );

    let distributor_after = svm.get_account(&distributor_pda).unwrap().data.clone();
    let claim_status_after = svm.get_account(&claim_status_pda).unwrap().data.clone();
    let vault_after = svm.get_account(&token_vault).unwrap().data.clone();
    let to_after = svm.get_account(&claimant_ata).unwrap().data.clone();
    let payer_after = svm.get_account(&claimant.pubkey()).unwrap().lamports;

    let storage_bytes_changed = bytes_changed(&distributor_before, &distributor_after)
        + bytes_changed(&claim_status_before, &claim_status_after)
        + bytes_changed(&vault_before, &vault_after)
        + bytes_changed(&to_before, &to_after);
    let fee_lamports = payer_before.saturating_sub(payer_after);

    let vault_state = SplAccount::unpack(&vault_after).unwrap();
    let to_state = SplAccount::unpack(&to_after).unwrap();
    assert_eq!(
        to_state.amount, amount_locked,
        "claimant should receive the locked tokens"
    );
    assert_eq!(
        vault_state.amount,
        vault_initial_amount - amount_locked,
        "vault should decrease by entire entitlement"
    );

    let mut cs_slice = claim_status_after.as_slice();
    let mut dist_slice = distributor_after.as_slice();
    let cs = ClaimStatus::try_deserialize(&mut cs_slice).expect("deserialize claim_status");
    let dist =
        MerkleDistributor::try_deserialize(&mut dist_slice).expect("deserialize distributor");
    assert_eq!(cs.locked_amount_withdrawn, amount_locked);
    assert_eq!(dist.total_amount_claimed, amount_locked);

    eprintln!(
        "use_fast_path={use_fast_path} storage_bytes_changed={} cu={} fee_lamports={}",
        storage_bytes_changed, meta.compute_units_consumed, fee_lamports
    );
}

#[test]
//use_fast_path=true storage_bytes_changed=12 cu=21463 fee_lamports=5000
fn test_claim_locked_fast_path() {
    run_claim_locked_flow(true);
}

//use_fast_path=false storage_bytes_changed=12 cu=22966 fee_lamports=5000
#[test]
fn test_claim_locked_anchor_path() {
    run_claim_locked_flow(false);
}
