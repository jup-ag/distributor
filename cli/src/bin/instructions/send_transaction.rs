use std::{io::Write, thread::sleep, time::Duration};

use anchor_client::solana_client::{self, rpc_config::RpcSendTransactionConfig};
use solana_rpc_client::rpc_client::{RpcClient, SerializableTransaction};
use solana_sdk::{commitment_config::CommitmentConfig, transaction::Transaction};

pub(crate) fn send_transaction(
  transaction: &Transaction,
  client: &RpcClient,
  send_client: &RpcClient,
) -> solana_client::client_error::Result<()> {
    let result = client.simulate_transaction(transaction)?;
    println!("Simulate result: {:#?}", result.value);
    if result.value.err.is_some() {
        return Err(result.value.err.unwrap().into());
    }
    println!("Attempt signature: {}", transaction.get_signature());
    let mut attempt_nb = 0;
    loop {
        attempt_nb += 1;
        print!("\rAttempt {attempt_nb}...");
        let _ = std::io::stdout().flush();

        if !client.is_blockhash_valid(
            transaction.get_recent_blockhash(),
            CommitmentConfig::processed(),
        )? {
            println!("Blockhash is expired...");
            break;
        }
        let signature = send_client.send_transaction_with_config(
            transaction,
            RpcSendTransactionConfig {
                skip_preflight: true,
                max_retries: Some(0),
                ..Default::default()
            },
        )?;

        sleep(Duration::from_millis(1600));
        let status = client
            .confirm_transaction_with_commitment(&signature, CommitmentConfig::processed())?;
        if status.value {
            println!("Transaction confirmed");
            break;
        }
    }
  Ok(())
}