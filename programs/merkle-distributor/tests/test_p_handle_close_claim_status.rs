use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_lang::AccountSerialize;
use litesvm::LiteSVM;
use merkle_distributor::{
    instruction as md_ix,
    state::{claim_status::ClaimStatus, pino_claim_status::{PinoClaimStatus, CLAIM_STATUS_DISC}},
    id as PROGRAM_ID,
};
use solana_sdk::{
    account::Account as SdkAccount,
    hash::Hash,
    instruction::Instruction,
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
mod common;
use common::PROGRAM_BYTES;

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

fn make_pino_claim_status(claimant: &Pubkey, admin: &Pubkey) -> Vec<u8> {
    let mut data = vec![0u8; PinoClaimStatus::TOTAL_LEN];
    data[..8].copy_from_slice(&CLAIM_STATUS_DISC);
    let mut cs = PinoClaimStatus::default();
    cs.claimant = *claimant;
    cs.locked_amount = 0;
    cs.locked_amount_withdrawn = 0;
    cs.unlocked_amount = 0;
    cs.set_closable(true);
    cs.admin = *admin;
    let body_bytes = unsafe {
        core::slice::from_raw_parts(
            (&cs as *const PinoClaimStatus) as *const u8,
            PinoClaimStatus::BODY_LEN,
        )
    };
    data[8..8 + PinoClaimStatus::BODY_LEN].copy_from_slice(body_bytes);
    data
}

fn make_borsh_claim_status(claimant: &Pubkey, admin: &Pubkey) -> Vec<u8> {
    let mut cs = ClaimStatus::default();
    cs.claimant = *claimant;
    cs.locked_amount = 0;
    cs.locked_amount_withdrawn = 0;
    cs.unlocked_amount = 0;
    cs.closable = true;
    cs.admin = *admin;
    let mut data = vec![0u8; ClaimStatus::LEN];
    {
        let mut slice: &mut [u8] = data.as_mut_slice();
        cs.try_serialize(&mut slice).expect("serialize claim status");
    }
    data
}

fn run_close_claim_status_flow(use_fast_path: bool) {
    let mut svm = LiteSVM::new();
    let program_id = PROGRAM_ID();
    svm.add_program(program_id, PROGRAM_BYTES);

    let admin = Keypair::new();
    let claimant = Keypair::new();

    svm.airdrop(&admin.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&claimant.pubkey(), 2_000_000_000).unwrap();

    let claim_status = Pubkey::new_unique();
    let data = if use_fast_path {
        make_pino_claim_status(&claimant.pubkey(), &admin.pubkey())
    } else {
        make_borsh_claim_status(&claimant.pubkey(), &admin.pubkey())
    };
    let lamports = svm.minimum_balance_for_rent_exemption(data.len());
    let cs_account = SdkAccount {
        lamports,
        data,
        owner: program_id,
        executable: false,
        rent_epoch: 0,
    };
    svm.set_account(claim_status, cs_account).expect("set claim status");

    let ix_data = if use_fast_path {
        md_ix::CloseClaimStatus {}.data()
    } else {
        md_ix::AnchorCloseClaimStatus {}.data()
    };
    let ix = Instruction {
        program_id,
        accounts: merkle_distributor::accounts::CloseClaimStatus {
            claim_status,
            claimant: claimant.pubkey(),
            admin: admin.pubkey(),
        }
        .to_account_metas(None),
        data: ix_data,
    };

    let cs_before = svm.get_account(&claim_status).unwrap();
    let claimant_before = svm.get_account(&claimant.pubkey()).unwrap().lamports;

    let meta = send_tx(&mut svm, &admin, &[&admin], &[ix]);

    let cs_after = svm.get_account(&claim_status).unwrap();
    let claimant_after = svm.get_account(&claimant.pubkey()).unwrap().lamports;
    assert_eq!(cs_after.owner, system_program::ID);
    assert_eq!(cs_after.lamports, 0);
    let rent_refund = claimant_after.saturating_sub(claimant_before);
    assert!(
        rent_refund > 0,
        "expected rent refund, got {rent_refund}"
    );

    eprintln!(
        "close_claim_status use_fast_path={use_fast_path} data_len={} cu={} rent_refund={rent_refund}",
        cs_before.data.len(),
        meta.compute_units_consumed,
    );
}

//close_claim_status use_fast_path=true data_len=104 cu=1499 rent_refund=1614720
#[test]
fn test_close_claim_status_fast_path() {
    run_close_claim_status_flow(true);
}

//close_claim_status use_fast_path=false data_len=104 cu=2999 rent_refund=1614720
#[test]
fn test_close_claim_status_anchor_path() {
    run_close_claim_status_flow(false);
}
