use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
};

use anchor_client::solana_sdk::signer::Signer;
use anchor_client::solana_sdk::{
    commitment_config::CommitmentLevel, compute_budget::ComputeBudgetInstruction,
    signature::Signature,
};
use anchor_spl::{
    associated_token::{
        get_associated_token_address_with_program_id, spl_associated_token_account,
    },
    token::spl_token,
    token_2022::spl_token_2022,
};

use crate::*;

pub fn parse_send_addresse(path: &PathBuf) -> Result<HashMap<String, String>> {
    let mut file = File::open(path)?;
    let mut data = String::new();

    file.read_to_string(&mut data).unwrap();
    if data.is_empty() {
        return Ok(HashMap::new());
    }
    let hashmap = serde_json::from_str(&data)?;
    Ok(hashmap)
}

fn send_for_batch_address(
    args: &Args,
    token_decimals: u8,
    users: Vec<(String, u64)>,
) -> Result<(Signature, Vec<(String, u64)>)> {
    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");
    let token_program_id = client.get_account(&args.mint)?.owner;

    let source_vault = get_associated_token_address_with_program_id(
        &keypair.pubkey(),
        &args.mint,
        &token_program_id,
    );

    let mut mass_ixs = vec![ComputeBudgetInstruction::set_compute_unit_limit(1_000_000)];

    // check priority fee
    if let Some(priority_fee) = args.priority_fee {
        mass_ixs.push(ComputeBudgetInstruction::set_compute_unit_price(
            priority_fee,
        ));
    }

    let mut qualified_users = vec![];
    for user in users.iter() {
        match Pubkey::from_str(&user.0) {
            Ok(user_address) => {
                qualified_users.push(user.clone());
                let user_ata =
                    spl_associated_token_account::get_associated_token_address_with_program_id(
                        &user_address,
                        &args.mint,
                        &token_program_id,
                    );

                if client.get_account_data(&user_ata).is_err() {
                    mass_ixs.push(
                        spl_associated_token_account::instruction::create_associated_token_account(
                            &keypair.pubkey(),
                            &user_address,
                            &args.mint,
                            &token_program_id,
                        ),
                    );
                }

                let amount = user.1;

                let transfer_ix = if token_program_id == spl_token_2022::id() {
                    // Use Token 2022 transfer instruction
                    spl_token_2022::instruction::transfer_checked(
                        &token_program_id,
                        &source_vault,
                        &args.mint,
                        &user_ata,
                        &keypair.pubkey(),
                        &[],
                        amount,
                        token_decimals,
                    )?
                } else {
                    // Use SPL Token transfer instruction
                    spl_token::instruction::transfer(
                        &token_program_id,
                        &source_vault,
                        &user_ata,
                        &keypair.pubkey(),
                        &[],
                        amount,
                    )?
                };

                mass_ixs.push(transfer_ix)
            }
            Err(_) => {
                println!("{} is not pubkey", user.0);
            }
        }
    }

    let tx = Transaction::new_signed_with_payer(
        &mass_ixs,
        Some(&keypair.pubkey()),
        &[&keypair],
        client.get_latest_blockhash().unwrap(),
    );

    let signature = client.send_transaction(&tx)?;
    Ok((signature, qualified_users))
}

pub fn process_mass_send(args: &Args, mass_send_args: &MassSendArgs) {
    let addresses = parse_new_record(&mass_send_args.csv_path).unwrap();

    let number_of_addresses_per_transaction = mass_send_args.max_address_per_tx as usize;

    let mut sent_addresses = parse_send_addresse(&mass_send_args.des_path).unwrap();

    println!("{:?}", sent_addresses);

    let mut index = 0;
    let mut batch_addresses = vec![];

    let mut wrap_batch_addresses = vec![];

    while index < addresses.len() {
        if let Some(_address) = sent_addresses.get(&addresses[index].0) {
            // this addresses has been sent, skip
            index += 1;
            continue;
        }
        batch_addresses.push(addresses[index].clone());
        if batch_addresses.len() >= number_of_addresses_per_transaction {
            wrap_batch_addresses.push(batch_addresses.clone());
            batch_addresses = vec![];
        }
        index += 1;
    }

    if batch_addresses.len() > 0 {
        wrap_batch_addresses.push(batch_addresses.clone());
    }
    println!(
        "num record {} {}",
        addresses.len(),
        wrap_batch_addresses.len()
    );

    for batch_address in wrap_batch_addresses.iter() {
        match send_for_batch_address(args, mass_send_args.token_decimals, batch_address.to_vec()) {
            Ok((signature, qualified_users)) => {
                println!("signature {}", signature);
                for user in qualified_users.iter() {
                    sent_addresses.insert(user.0.clone(), signature.to_string());
                }

                // write it again to cache
                let serialized = serde_json::to_string_pretty(&sent_addresses).unwrap();
                let mut file: File = File::create(&mass_send_args.des_path).unwrap();
                file.write_all(serialized.as_bytes()).unwrap();
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }
}

pub fn process_resend(args: &Args, resend_args: &ResendSendArgs) {
    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());
    let mut sent_addresses = parse_send_addresse(&resend_args.des_path).unwrap();
    let original_datasets = parse_new_record(&resend_args.csv_path).unwrap();

    let users: HashMap<String, u64> = original_datasets
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();

    let mut signature_to_address: HashMap<String, Vec<String>> = HashMap::new();

    for (key, value) in sent_addresses.iter() {
        if let Some(array_addr) = signature_to_address.get_mut(value) {
            array_addr.push(key.clone());
            // signature_to_address.insert(value, array_addr.clone());
        } else {
            signature_to_address.insert(value.clone(), vec![key.clone()]);
        }
    }

    let number_of_addresses_per_transaction = resend_args.max_address_per_tx as usize;

    for (signature, addresses) in signature_to_address.iter() {
        match client.get_signature_status_with_commitment_and_history(
            &Signature::from_str(signature).unwrap(),
            CommitmentConfig {
                commitment: CommitmentLevel::Finalized,
            },
            true,
        ) {
            Ok(value) => {
                let mut should_resend = false;
                if value.is_none() {
                    println!("{} is not existed resend", signature);
                    should_resend = true;
                } else {
                    match value.unwrap() {
                        Ok(_) => {}
                        Err(err) => {
                            println!("{} is error {}", signature, err);
                            should_resend = true;
                        }
                    }
                }

                if should_resend {
                    let should_send_addresses =
                        if addresses.len() > number_of_addresses_per_transaction {
                            // break it to double, or ignore
                            addresses[0..number_of_addresses_per_transaction].to_vec()
                        } else {
                            addresses.clone()
                        };

                    let resend_users_with_amount: Vec<(String, u64)> = should_send_addresses
                        .iter()
                        .filter_map(|key| users.get(key).map(|&value| (key.clone(), value)))
                        .collect();

                    match send_for_batch_address(
                        args,
                        resend_args.token_decimals,
                        resend_users_with_amount,
                    ) {
                        Ok((signature, qualified_address)) => {
                            println!("signature {}", signature);
                            for address in qualified_address.iter() {
                                sent_addresses.insert(address.0.clone(), signature.to_string());
                            }

                            let serialized =
                                serde_json::to_string_pretty(&sent_addresses.clone()).unwrap();
                            let mut file: File = File::create(&resend_args.des_path).unwrap();
                            file.write_all(serialized.as_bytes()).unwrap();
                        }
                        Err(err) => {
                            println!("{}", err);
                        }
                    }
                }
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }
}
