use clap::{Arg, Command};
use csv::Writer;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::{Keypair, Signer};
use std::fs::File;

#[derive(Serialize, Deserialize)]
struct TestWallet {
    pubkey: String,
    amount: f64,
    keypair: String,
}

#[derive(Serialize, Deserialize)]
struct TestWalletData {
    generated_at: String,
    total_wallets: usize,
    total_tokens: f64,
    wallets: Vec<TestWallet>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Test Wallet Generator")
        .version("1.0")
        .about("Generates test wallets with fixed amounts for production testing")
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .value_name("NUMBER")
                .help("Number of test wallets to generate")
                .default_value("50"),
        )
        .arg(
            Arg::new("min-amount")
                .long("min-amount")
                .value_name("TOKENS")
                .help("Minimum token amount (caps fixed amounts below this)")
                .default_value("0.01"),
        )
        .arg(
            Arg::new("max-amount")
                .long("max-amount")
                .value_name("TOKENS")
                .help("Maximum token amount (caps fixed amounts above this)")
                .default_value("3"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output JSON file for private keys")
                .default_value("test_wallets_CONFIDENTIAL.json"),
        )
        .arg(
            Arg::new("csv")
                .long("csv")
                .value_name("FILE")
                .help("Output CSV file for recipients")
                .default_value("test_recipients.csv"),
        )
        .get_matches();

    let count: usize = matches.get_one::<String>("count").unwrap().parse()?;
    let min_amount: f64 = matches.get_one::<String>("min-amount").unwrap().parse()?;
    let max_amount: f64 = matches.get_one::<String>("max-amount").unwrap().parse()?;
    let output_file = matches.get_one::<String>("output").unwrap();
    let csv_file = matches.get_one::<String>("csv").unwrap();

    println!("🔑 Generating {} test wallets...", count);
    println!(
        "💰 Amount range: {} - {} WCT (using fixed amounts)",
        min_amount, max_amount
    );

    let mut wallets = Vec::new();
    let mut total_tokens = 0.0;

    // Define fixed amounts for each category
    let small_amounts = vec![0.01, 0.02, 0.05, 0.07, 0.08];
    let medium_amounts = vec![0.1, 0.25, 0.5, 0.75, 0.9];
    let large_amounts = vec![1.0, 1.5, 2.0, 2.5, 3.0];

    // Create CSV writer
    let file = File::create(csv_file)?;
    let mut wtr = Writer::from_writer(file);
    wtr.write_record(&["pubkey", "amount", "locked_amount"])?;

    // Generate wallets with fixed amounts
    for i in 0..count {
        let keypair = Keypair::new();
        let pubkey = keypair.pubkey().to_string();

        // Generate amount with fixed distribution:
        // 60% get small amounts (0.01 - 0.08)
        // 30% get medium amounts (0.1 - 0.9)
        // 10% get larger amounts (1.0 - 3.0)
        let amount = if i < (count * 60 / 100) {
            small_amounts[i % small_amounts.len()]
        } else if i < (count * 90 / 100) {
            medium_amounts[(i - (count * 60 / 100)) % medium_amounts.len()]
        } else {
            large_amounts[(i - (count * 90 / 100)) % large_amounts.len()]
        };

        // Ensure amount is within user-specified range
        let amount = if amount < min_amount {
            min_amount
        } else if amount > max_amount {
            max_amount
        } else {
            amount
        };

        // Round to 3 decimal places
        let amount = (amount * 1000.0).round() / 1000.0;
        total_tokens += amount;

        // Write to CSV
        wtr.write_record(&[&pubkey, &amount.to_string(), "0"])?;

        // Store wallet data
        let private_key_bytes = keypair.to_bytes();
        let private_key_base58 = bs58::encode(&private_key_bytes).into_string();

        wallets.push(TestWallet {
            pubkey: pubkey.clone(),
            amount,
            keypair: private_key_base58,
        });

        if (i + 1) % 10 == 0 {
            println!("Progress: {}/{}", i + 1, count);
        }
    }

    wtr.flush()?;

    // Sort wallets by amount (descending) for easier testing
    wallets.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap());

    // Create secure JSON file with all data
    let wallet_data = TestWalletData {
        generated_at: chrono::Utc::now().to_rfc3339(),
        total_wallets: count,
        total_tokens: (total_tokens * 1000.0).round() / 1000.0,
        wallets,
    };

    let json_file = File::create(output_file)?;
    serde_json::to_writer_pretty(json_file, &wallet_data)?;

    // Summary
    println!("\n✅ Generated {} test wallets", count);
    println!("💰 Total tokens allocated: {:.3} WCT", total_tokens);
    println!("🔢 Using fixed amounts: Small [0.01,0.02,0.05,0.07,0.08], Medium [0.1,0.25,0.5,0.75,0.9], Large [1.0,1.5,2.0,2.5,3.0]");
    println!("📄 CSV saved to: {}", csv_file);
    println!("🔐 Private keys saved to: {}", output_file);
    println!("\n⚠️  SECURITY WARNINGS:");
    println!("   - Keep {} secure!", output_file);
    println!("   - This file contains private keys!");
    println!("   - Set file permissions: chmod 600 {}", output_file);
    println!("   - Delete after testing is complete");

    // Show distribution
    println!("\n📊 Amount distribution:");
    let small_count = wallet_data
        .wallets
        .iter()
        .filter(|w| w.amount < 0.1)
        .count();
    let medium_count = wallet_data
        .wallets
        .iter()
        .filter(|w| w.amount >= 0.1 && w.amount < 1.0)
        .count();
    let large_count = wallet_data
        .wallets
        .iter()
        .filter(|w| w.amount >= 1.0)
        .count();

    println!("   Small (0.01-0.08 WCT): {} wallets", small_count);
    println!("   Medium (0.1-0.9 WCT): {} wallets", medium_count);
    println!("   Large (1.0-3.0 WCT): {} wallets", large_count);

    // Show top 5 wallets
    println!("\n🏆 Top 5 test wallets:");
    for (i, wallet) in wallet_data.wallets.iter().take(5).enumerate() {
        println!(
            "   {}. {} - {:.3} WCT",
            i + 1,
            &wallet.pubkey[..8],
            wallet.amount
        );
    }

    Ok(())
}
