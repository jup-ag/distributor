use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::token::spl_token::solana_program::program_pack::Pack;
use litesvm::LiteSVM;
use solana_sdk::{
    hash::Hash,
    instruction::Instruction,
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};


use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_associated_token_account::solana_program::program_option::COption;
use spl_token::state::{Account as SplAccount, AccountState as SplAccountState, Mint as SplMint};
use spl_token::{id as spl_token_program_id};

use merkle_distributor::accounts as md_accounts;
use merkle_distributor::instruction as md_ix;
use merkle_distributor::{self, id as PROGRAM_ID};
const PROGRAM_BYTES: &[u8] = include_bytes!("../../../target/deploy/merkle_distributor.so");


/// --- Helpers to pack SPL data into account bytes ---------------------------------------------

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
    let acc = solana_sdk::account::Account {
        lamports: min_lamports,
        data,
        owner: *owner,
        executable: false,
        rent_epoch: 0,
    };
    svm.set_account(*address, acc).expect("set_account");
}

#[test]
fn test_new_distributor_ok() {
    let mut svm = LiteSVM::new();
    let program_id = PROGRAM_ID();
    svm.add_program(program_id, PROGRAM_BYTES);

    let admin = Keypair::new();
    let admin_pk = admin.pubkey();

    svm.airdrop(&admin_pk, 2_000_000_000).expect("airdrop");

    // --- Create SPL Mint
    let mint = Pubkey::new_unique();
    let mint_data = pack_mint(&admin_pk, 6, 0);
    svm_set_owned_with_data(&mut svm, &mint, &spl_token_program_id(), mint_data);
    
    // --- Create clawback_receiver token account (owner = admin, program owner = spl_token::ID)
    let clawback_receiver = Pubkey::new_unique();
    let clawback_data = pack_token_account(&mint, &admin_pk, 0);
    svm_set_owned_with_data(
        &mut svm,
        &clawback_receiver,
        &spl_token_program_id(),
        clawback_data,
    );

    // Seeds = ["MerkleDistributor", base, mint, version-le] 
    let base = Keypair::new();
    let base_pk = base.pubkey();
    let version: u64 = 1;

    let (distributor_pda, _bump) = Pubkey::find_program_address(
        &[
            b"MerkleDistributor",
            base_pk.as_ref(),
            mint.as_ref(),
            &version.to_le_bytes(),
        ],
        &program_id,
    );

    // token_vault = ATA(distributor_pda, mint)
    let token_vault =
        get_associated_token_address_with_program_id(&distributor_pda, &mint, &spl_token_program_id());

    // authority/owner = distributor PDA
    let vault_data = pack_token_account(&mint, &distributor_pda, 0);
    svm_set_owned_with_data(&mut svm, &token_vault, &spl_token_program_id(), vault_data);

    // --- Build accounts for `new_distributor` 
    let accounts = md_accounts::NewDistributor {
        distributor: distributor_pda,
        base: base_pk,
        clawback_receiver,
        mint,
        token_vault,
        admin: admin_pk,
        system_program: system_program::ID,
        associated_token_program: spl_associated_token_account::ID, // v6 constant
        token_program: spl_token_program_id(),
    };


    let root = [7u8; 32];
    let start_vesting_ts: i64 = 4_000_000_000; // future
    let end_vesting_ts: i64 = start_vesting_ts + 10;
    let clawback_start_ts: i64 = end_vesting_ts + 24 * 3600 + 10; // ≥ 1 day after end

    let max_total_claim: u64 = 1_000_000;
    let max_num_nodes: u64 = 1;
    let activation_point: u64 = 0;
    let activation_type: u8 = 0; 
    let closable: bool = true;

    // --- Instruction 
    let ix = Instruction {
        program_id,
        accounts: accounts.to_account_metas(None),
        data: md_ix::NewDistributor {
            version,
            root,
            max_total_claim,
            max_num_nodes,
            start_vesting_ts,
            end_vesting_ts,
            clawback_start_ts,
            activation_point,
            activation_type,
            closable,
        }
            .data(),
    };

    // --- Send tx (admin & base must sign)
    let bh: Hash = svm.latest_blockhash();
    let msg = Message::new(&[ix], Some(&admin_pk));
    let tx = Transaction::new(&[&admin, &base], msg, bh);
    let meta = svm.send_transaction(tx).expect("send new_distributor");

    assert!(meta.compute_units_consumed > 0, "tx consumed 0 CU: {:?}", meta);
    let logs = meta.logs.join("\n");
    assert!(
        logs.contains("New distributor created") || logs.contains("New distributor"),
        "expected creation log; logs={logs}"
    );
}
