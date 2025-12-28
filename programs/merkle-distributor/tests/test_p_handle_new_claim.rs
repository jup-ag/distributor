
use anchor_lang::solana_program::program_option::COption;
use anchor_lang::solana_program::program_pack::Pack;
use anchor_lang::{InstructionData, ToAccountMetas};
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
use spl_token::state::{Account as SplAccount, AccountState as SplAccountState, Mint as SplMint};
use spl_token::{id as spl_token_program_id};

use merkle_distributor::accounts as md_accounts;
use merkle_distributor::instruction as md_ix;
use merkle_distributor::{self, id as PROGRAM_ID};

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

fn run_new_claim_flow(use_fast_path: bool) {
    println!("starting tests");
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

    let version: u64 = 1;
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

    let start_vesting_ts: i64 = 4_000_000_000;
    let end_vesting_ts: i64 = start_vesting_ts + 10;
    let clawback_start_ts: i64 = end_vesting_ts + 24 * 3600 + 10;

    let amount_unlocked: u64 = 123_000;
    let amount_locked: u64 = 456_000;

    let leaf_inner = anchor_lang::solana_program::hash::hashv(&[
        claimant.pubkey().as_ref(),
        &amount_unlocked.to_le_bytes(),
        &amount_locked.to_le_bytes(),
    ]);
    const LEAF_PREFIX: &[u8] = &[0];
    let leaf = anchor_lang::solana_program::hash::hashv(&[LEAF_PREFIX, &leaf_inner.to_bytes()]);
    let root: [u8; 32] = leaf.to_bytes();

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
        data: if use_fast_path {
            md_ix::NewDistributor {
                version,
                root,
                max_total_claim: 1_000_000,
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
                root,
                max_total_claim: 1_000_000,
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
    let init_meta = send_tx(&mut svm, &admin, &[&admin, &base], &[ix_new]);
    let init_logs = init_meta.logs.join("\n");
    let expected = if use_fast_path {
        "New distributor (fast)"
    } else {
        "New distributor created"
    };
    assert!(
        init_logs.contains(expected),
        "expected distributor init log '{expected}', got: {init_logs}"
    );

    let proof: Vec<[u8; 32]> = vec![];
    let (claim_status_pda, _cs_bump) = Pubkey::find_program_address(
        &[
            b"ClaimStatus",
            claimant.pubkey().as_ref(),
            distributor_pda.as_ref(),
        ],
        &program_id,
    );

    if use_fast_path {
        let cs_len = merkle_distributor::state::pino_claim_status::PinoClaimStatus::TOTAL_LEN;
        let cs_data = vec![0u8; cs_len];
        let min_lamports = svm.minimum_balance_for_rent_exemption(cs_len);
        let cs_acc = solana_sdk::account::Account {
            lamports: min_lamports,
            data: cs_data,
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        };
        svm.set_account(claim_status_pda, cs_acc)
            .expect("set claim_status");
    }

    let ix_data = if use_fast_path {
        md_ix::NewClaim {
            amount_unlocked,
            amount_locked,
            proof: proof.clone(),
        }
            .data()
    } else {
        md_ix::AnchorNewClaim {
            amount_unlocked,
            amount_locked,
            proof: proof.clone(),
        }
            .data()
    };

    let metas = vec![
        AccountMeta::new(distributor_pda, false),
        AccountMeta::new(claim_status_pda, false),
        AccountMeta::new(token_vault, false),
        AccountMeta::new(claimant_ata, false),
        AccountMeta::new(claimant.pubkey(), true),
        AccountMeta::new_readonly(spl_token_program_id(), false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    let ix_claim = Instruction {
        program_id,
        accounts: metas,
        data: ix_data,
    };

    let meta = send_tx(&mut svm, &admin, &[&admin, &claimant], &[ix_claim]);
    assert!(meta.compute_units_consumed > 0, "claim tx consumed 0 CU: {:?}", meta);
    let logs = meta.logs.join("\n");
    assert!(logs.contains("claim"), "expected claim log; logs={logs}");

    let vault_acc = svm.get_account(&token_vault).unwrap();
    let to_acc = svm.get_account(&claimant_ata).unwrap();
    let vault_state = SplAccount::unpack(&vault_acc.data).unwrap();
    let to_state = SplAccount::unpack(&to_acc.data).unwrap();
    assert!(vault_state.amount < vault_initial_amount, "vault did not decrease");
    assert!(to_state.amount > 0, "claimant did not receive tokens");

    eprintln!(
        "use_fast_path={use_fast_path} init_cu={} claim_cu={}",
        init_meta.compute_units_consumed, meta.compute_units_consumed
    );
}

#[test]
//use_fast_path=true init_cu=29265 claim_cu=18194
fn test_handle_new_claim_happy_path_fast() {
    run_new_claim_flow(true);
}

//use_fast_path=false init_cu=50941 claim_cu=30964
#[test]
fn test_handle_new_claim_happy_path_anchor() {
    run_new_claim_flow(false);
}
