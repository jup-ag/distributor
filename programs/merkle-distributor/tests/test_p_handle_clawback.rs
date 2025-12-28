use anchor_lang::{
    solana_program::{program_option::COption, program_pack::Pack},
    InstructionData, ToAccountMetas,
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

fn run_clawback_flow(use_fast_path: bool) {
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

    let version: u64 = 42;
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
    let vault_initial_amount: u64 = 999_999;
    let vault_data = pack_token_account(&mint, &distributor_pda, 0);
    svm_set_owned_with_data(&mut svm, &token_vault, &spl_token_program_id(), vault_data);

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
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = end_vesting_ts + 24 * 3600 + 2;
    let clawback_start_ts = clock.unix_timestamp;

    let ix_new = Instruction {
        program_id,
        accounts: accounts_new.to_account_metas(None),
        data: if use_fast_path {
            md_ix::NewDistributor {
                version,
                root: [7u8; 32],
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
                root: [7u8; 32],
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

    // Seed vault balance now that distributor exists.
    let vault_data = pack_token_account(&mint, &distributor_pda, vault_initial_amount);
    svm_set_owned_with_data(&mut svm, &token_vault, &spl_token_program_id(), vault_data);

    svm.set_sysvar::<Clock>(&clock);

    let ix_data = if use_fast_path {
        md_ix::Clawback {}.data()
    } else {
        md_ix::AnchorClawback {}.data()
    };
    let clawback_accounts = vec![
        AccountMeta::new(distributor_pda, false),
        AccountMeta::new(token_vault, false),
        AccountMeta::new(clawback_receiver, false),
        AccountMeta::new(claimant.pubkey(), true),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new_readonly(spl_token_program_id(), false),
    ];
    let ix_clawback = Instruction {
        program_id,
        accounts: clawback_accounts,
        data: ix_data,
    };

    let distributor_before = svm.get_account(&distributor_pda).unwrap().data.clone();
    let vault_before = svm.get_account(&token_vault).unwrap().data.clone();
    let clawback_before = svm.get_account(&clawback_receiver).unwrap().data.clone();
    let payer_before = svm.get_account(&claimant.pubkey()).unwrap().lamports;

    let meta = send_tx(&mut svm, &claimant, &[&claimant], &[ix_clawback]);
    let distributor_after = svm.get_account(&distributor_pda).unwrap().data.clone();
    let vault_after = svm.get_account(&token_vault).unwrap().data.clone();
    let clawback_after = svm.get_account(&clawback_receiver).unwrap().data.clone();
    let payer_after = svm.get_account(&claimant.pubkey()).unwrap().lamports;

    let storage_bytes_changed = bytes_changed(&distributor_before, &distributor_after)
        + bytes_changed(&vault_before, &vault_after)
        + bytes_changed(&clawback_before, &clawback_after);
    let fee_lamports = payer_before.saturating_sub(payer_after);

    let vault_state = SplAccount::unpack(&vault_after).unwrap();
    let clawback_state = SplAccount::unpack(&clawback_after).unwrap();
    assert_eq!(vault_state.amount, 0, "vault not emptied");
    assert_eq!(
        clawback_state.amount, vault_initial_amount,
        "clawback receiver missing funds"
    );

    eprintln!(
        "clawback use_fast_path={use_fast_path} storage_bytes_changed={} cu={} fee_lamports={}",
        storage_bytes_changed, meta.compute_units_consumed, fee_lamports
    );
}

//clawback use_fast_path=true storage_bytes_changed=7 cu=9565 fee_lamports=5000
#[test]
fn test_clawback_fast_path() {
    run_clawback_flow(true);
}

//clawback use_fast_path=false storage_bytes_changed=7 cu=18947 fee_lamports=5000
#[test]
fn test_clawback_anchor_path() {
    run_clawback_flow(false);
}
