use anchor_lang::{
    solana_program::{program_option::COption, program_pack::Pack},
    InstructionData, ToAccountMetas,
};
use litesvm::LiteSVM;
use solana_sdk::{
    account::Account as SdkAccount,
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

use merkle_distributor::{accounts as md_accounts, instruction as md_ix, id as PROGRAM_ID};

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

fn run_close_distributor_flow(use_fast_path: bool) {
    let mut svm = LiteSVM::new();
    let program_id = PROGRAM_ID();
    svm.add_program(program_id, PROGRAM_BYTES);

    let admin = Keypair::new();
    let base = Keypair::new();

    svm.airdrop(&admin.pubkey(), 2_000_000_000).unwrap();

    let mint = Pubkey::new_unique();
    let mint_data = pack_mint(&admin.pubkey(), 6, 0);
    svm_set_owned_with_data(&mut svm, &mint, &spl_token_program_id(), mint_data);

    let version: u64 = 55;
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
    svm_set_owned_with_data(
        &mut svm,
        &token_vault,
        &spl_token_program_id(),
        pack_token_account(&mint, &distributor_pda, 0),
    );

    let destination_token_account = get_associated_token_address_with_program_id(
        &admin.pubkey(),
        &mint,
        &spl_token_program_id(),
    );
    svm_set_owned_with_data(
        &mut svm,
        &destination_token_account,
        &spl_token_program_id(),
        pack_token_account(&mint, &admin.pubkey(), 0),
    );

    let clawback_receiver = Pubkey::new_unique();
    let clawback_data = pack_token_account(&mint, &admin.pubkey(), 0);
    svm_set_owned_with_data(&mut svm, &clawback_receiver, &spl_token_program_id(), clawback_data);

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

    let start_vesting_ts: i64 = 4_000_000_000;
    let end_vesting_ts: i64 = start_vesting_ts + 10;
    let clawback_start_ts: i64 = end_vesting_ts + 24 * 3600 + 10;

    let ix_new = Instruction {
        program_id,
        accounts: accounts_new.to_account_metas(None),
        data: if use_fast_path {
            md_ix::NewDistributor {
                version,
                root: [9u8; 32],
                max_total_claim: 5_000_000,
                max_num_nodes: 10,
                start_vesting_ts,
                end_vesting_ts,
                clawback_start_ts,
                activation_point: 0,
                activation_type: 0,
                closable: true,
            }
                .data()
        } else {
            md_ix::AnchorNewDistributor {
                version,
                root: [9u8; 32],
                max_total_claim: 5_000_000,
                max_num_nodes: 10,
                start_vesting_ts,
                end_vesting_ts,
                clawback_start_ts,
                activation_point: 0,
                activation_type: 0,
                closable: true,
            }
                .data()
        },
    };
    send_tx(&mut svm, &admin, &[&admin, &base], &[ix_new]);

    // Fund the vault after initialization.
    let vault_initial_amount: u64 = 321_000;
    svm_set_owned_with_data(
        &mut svm,
        &token_vault,
        &spl_token_program_id(),
        pack_token_account(&mint, &distributor_pda, vault_initial_amount),
    );

    let ix_data = if use_fast_path {
        md_ix::CloseDistributor {}.data()
    } else {
        md_ix::AnchorCloseDistributor {}.data()
    };
    let ix_close = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(distributor_pda, false),
            AccountMeta::new(token_vault, false),
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(destination_token_account, false),
            AccountMeta::new_readonly(spl_token_program_id(), false),
        ],
        data: ix_data,
    };

    let distributor_before = svm.get_account(&distributor_pda).unwrap();
    let vault_before = svm.get_account(&token_vault).unwrap();
    let destination_before = svm.get_account(&destination_token_account).unwrap();
    let admin_before = svm.get_account(&admin.pubkey()).unwrap().lamports;

    let meta = send_tx(&mut svm, &admin, &[&admin], &[ix_close]);

    let distributor_after = svm.get_account(&distributor_pda).unwrap();
    let vault_after = svm.get_account(&token_vault).unwrap();
    let destination_after = svm.get_account(&destination_token_account).unwrap();
    let admin_after = svm.get_account(&admin.pubkey()).unwrap().lamports;

    let storage_bytes_changed = bytes_changed(&distributor_before.data, &distributor_after.data)
        + bytes_changed(&vault_before.data, &vault_after.data)
        + bytes_changed(&destination_before.data, &destination_after.data);

    let fee_lamports = admin_before.saturating_sub(admin_after);
    let vault_state = SplAccount::unpack(&vault_after.data).unwrap();
    let dest_state = SplAccount::unpack(&destination_after.data).unwrap();
    assert_eq!(vault_state.amount, 0, "vault not emptied");
    assert_eq!(
        dest_state.amount, vault_initial_amount,
        "destination missing funds"
    );
    assert_eq!(
        distributor_after.owner,
        system_program::ID,
        "distributor not closed"
    );
    assert_eq!(distributor_after.lamports, 0, "distributor lamports remain");

    eprintln!(
        "close_distributor use_fast_path={use_fast_path} storage_bytes_changed={} cu={} fee_lamports={fee_lamports}",
        storage_bytes_changed, meta.compute_units_consumed
    );
}

//close_distributor use_fast_path=true storage_bytes_changed=162 cu=9137 fee_lamports=0
#[test]
fn test_close_distributor_fast_path() {
    run_close_distributor_flow(true);
}

//close_distributor use_fast_path=false storage_bytes_changed=382 cu=13809 fee_lamports=0
#[test]
fn test_close_distributor_anchor_path() {
    run_close_distributor_flow(false);
}
