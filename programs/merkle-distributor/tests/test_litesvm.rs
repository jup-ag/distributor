
#![allow(unused_imports)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::let_and_return)]

use litesvm::LiteSVM;

use solana_sdk::{
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
// If your names differ, tweak these three lines.
use merkle_distributor::accounts as md_accounts;
use merkle_distributor::instruction as md_ix;
use merkle_distributor::{self, id as PROGRAM_ID};

const PROGRAM_BYTES: &[u8] = include_bytes!("../../../target/deploy/merkle_distributor.so");

// fn to_address(pk: &Pubkey) -> Address {
//     Address::new_from_array(pk.to_bytes())
// }
#[test]
fn test_svm_deploy() {
    let mut svm=   LiteSVM::new();
    let program_id = PROGRAM_ID();
    (svm).add_program(program_id, PROGRAM_BYTES);
    // --- Payer / authority
    let payer = Keypair::new();
    let payer_pk = payer.pubkey();

    // Fund payer (LiteSVM 0.6.1 expects &Pubkey here)
    svm.airdrop(&payer_pk, 1_000_000_000).expect("airdrop failed");


}
// 
// #[test]
// fn test_new_distributor_with_positional_args() {
//     // --- Boot test VM and load program
//     let mut svm = LiteSVM::new();
//     let program_id = PROGRAM_ID();
//     svm.add_program(program_id, PROGRAM_BYTES);
// 
//     // --- Payer / authority
//     let payer = Keypair::new();
//     let payer_pk: Pubkey = payer.pubkey();
// 
//     // Fund payer (LiteSVM 0.6.1: &Pubkey)
//     svm.airdrop(&payer_pk, 1_000_000_000).expect("airdrop failed");
// 
//     // --- Accounts your Context<NewDistributor> likely needs
//     // Adjust if your accounts struct differs.
//     let distributor = Pubkey::new_unique();
// 
//     let accounts = md_accounts::NewDistributor {
//         distributor,
//         base: Default::default(),
//         clawback_receiver: Default::default(),
//         mint: Default::default(),
//         token_vault: Default::default(),
//         admin: Default::default(),
//         system_program: Default::default(),
//         associated_token_program: Default::default(),
//         token_program: Default::default(),
//     };
// 
//     // --- Positional args exactly match your function signature:
//     // pub fn new_distributor(
//     //   ctx: Context<NewDistributor>,
//     //   version: u64,
//     //   root: [u8; 32],
//     //   max_total_claim: u64,
//     //   max_num_nodes: u64,
//     //   start_vesting_ts: i64,
//     //   end_vesting_ts: i64,
//     //   clawback_start_ts: i64,
//     //   activation_point: u64,
//     //   activation_type: u8,
//     //   closable: bool,
//     // ) -> Result<()>
//     let version: u64 = 0;
//     let root: [u8; 32] = [7u8; 32];
//     let max_total_claim: u64 = 1_000_000;
//     let max_num_nodes: u64 = 1;
//     let start_vesting_ts: i64 = 1;
//     let end_vesting_ts: i64 = 10;
//     let clawback_start_ts: i64 = 11;
//     let activation_point: u64 = 0; // set as needed
//     let activation_type: u8 = 0;   // 0=slot/ts per your program logic
//     let closable: bool = true;
// 
//     // Anchor generates an instruction struct named after your function:
//     let ix = solana_sdk::instruction::Instruction {
//         program_id,
//         accounts: accounts.to_account_metas(None),
//         data: md_ix::NewDistributor {
//             version,
//             root,
//             max_total_claim,
//             max_num_nodes,
//             start_vesting_ts,
//             end_vesting_ts,
//             clawback_start_ts,
//             activation_point,
//             activation_type,
//             closable,
//         }
//             .data(), // provided by Anchor's InstructionData
//     };
//     
//     
// 
//     // --- Build and send tx
//     let blockhash = svm.latest_blockhash();
//     let msg = Message::new(&[ix], Some(&payer_pk));
//     let tx = Transaction::new(&[&payer], msg, blockhash);
// 
//     let meta = svm.send_transaction(tx).expect("send new_distributor");
// 
//     // Sanity checks
//     assert!(
//         meta.compute_units_consumed > 0,
//         "init tx consumed 0 CU; meta={:?}",
//         meta
//     );
// 
//     let logs = meta.logs.join("\n");
//     // If your program logs something on success, assert it:
//     // e.g., assert!(logs.contains("initialized distributor"));
//     // For now just ensure it ran:
//     assert!(
//         !logs.is_empty(),
//         "no logs from program; meta={:?}",
//         meta
//     );
// }