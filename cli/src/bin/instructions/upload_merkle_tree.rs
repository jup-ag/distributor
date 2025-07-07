use crate::Args;
use anyhow::{Context, Result};
use bs58;
use hex;
use jito_merkle_tree::{airdrop_merkle_tree::AirdropMerkleTree, utils::get_merkle_distributor_pda};
use solana_program::pubkey::Pubkey;
use std::{fs, path::PathBuf, time::Duration};
use tokio_postgres::{Client, Error as PgError, NoTls};

/// Arguments for uploading merkle tree to PostgreSQL
#[derive(clap::Parser, Debug, Clone)]
pub struct UploadMerkleTreeArgs {
    /// Path to the folder containing merkle tree JSON files
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,

    /// PostgreSQL connection string (e.g., "host=localhost user=postgres dbname=merkle_trees password=secret")
    #[clap(long, env)]
    pub postgres_url: String,

    /// Table name to store the merkle tree data
    #[clap(long, env, default_value = "airdrop_recipients")]
    pub table_name: String,
}

/// Pre-computed record data for bulk insert
#[derive(Debug)]
struct PreparedRecord {
    recipient_base58: String,
    amount_hex: String,
    proof_hex_array: Vec<String>,
}

/// Validated merkle tree with metadata
#[derive(Debug)]
struct ValidatedMerkleTree {
    merkle_tree: AirdropMerkleTree,
    file_path: PathBuf,
    distributor_pda: Pubkey,
    distributor_index: String,
}

/// Process the upload of merkle trees from a folder to PostgreSQL database
pub async fn process_upload_merkle_tree(
    args: &Args,
    upload_args: &UploadMerkleTreeArgs,
) -> Result<()> {
    let start_time = std::time::Instant::now();
    println!(
        "Processing merkle trees from folder: {:?}",
        upload_args.merkle_tree_path
    );

    // Validate that the path is a directory
    if !upload_args.merkle_tree_path.is_dir() {
        return Err(anyhow::anyhow!(
            "Path {:?} is not a directory. Please provide a folder containing JSON files.",
            upload_args.merkle_tree_path
        ));
    }

    // Load and validate all JSON files in the folder
    println!("Scanning folder for JSON files...");
    let validated_trees = load_and_validate_merkle_trees(args, upload_args).await?;

    if validated_trees.is_empty() {
        return Err(anyhow::anyhow!(
            "No valid merkle tree JSON files found in folder: {:?}",
            upload_args.merkle_tree_path
        ));
    }

    println!(
        "Found {} valid merkle tree files to process",
        validated_trees.len()
    );

    // Process each validated merkle tree with fresh connection per file
    let mut total_records_processed = 0;
    let mut successful_uploads = 0;
    let mut failed_uploads = 0;

    for (index, validated_tree) in validated_trees.iter().enumerate() {
        println!(
            "\n=== Processing file {}/{}: {} ===",
            index + 1,
            validated_trees.len(),
            validated_tree
                .file_path
                .file_name()
                .unwrap()
                .to_string_lossy()
        );

        // Create fresh connection for each file to avoid connection issues
        match create_db_connection(&upload_args.postgres_url).await {
            Ok(mut client) => {
                match process_single_merkle_tree(
                    &mut client,
                    validated_tree,
                    &upload_args.table_name,
                )
                .await
                {
                    Ok(records_uploaded) => {
                        successful_uploads += 1;
                        total_records_processed += records_uploaded;
                        if records_uploaded == 0 {
                            println!(
                                "✅ Skipped {} (all records already exist)",
                                validated_tree
                                    .file_path
                                    .file_name()
                                    .unwrap()
                                    .to_string_lossy()
                            );
                        } else {
                            println!(
                                "✅ Successfully uploaded {} new records from {}",
                                records_uploaded,
                                validated_tree
                                    .file_path
                                    .file_name()
                                    .unwrap()
                                    .to_string_lossy()
                            );
                        }
                    }
                    Err(e) => {
                        failed_uploads += 1;
                        eprintln!(
                            "❌ Failed to upload {}: {}",
                            validated_tree
                                .file_path
                                .file_name()
                                .unwrap()
                                .to_string_lossy(),
                            e
                        );
                        // Continue with other files even if one fails
                    }
                }
            }
            Err(e) => {
                failed_uploads += 1;
                eprintln!(
                    "❌ Failed to connect to database for {}: {}",
                    validated_tree
                        .file_path
                        .file_name()
                        .unwrap()
                        .to_string_lossy(),
                    e
                );
            }
        }

        // Add longer delay between files to prevent connection issues
        if index < validated_trees.len() - 1 {
            println!("  💤 Waiting 2 seconds before processing next file...");
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
    // Final summary
    println!("\n=== Upload Summary ===");
    println!("✅ Successful files: {}", successful_uploads);
    println!("❌ Failed files: {}", failed_uploads);
    println!("📊 Total new records uploaded: {}", total_records_processed);
    println!("🕒 Total time: {:.2?}", start_time.elapsed());
    println!("📁 Table: {}", upload_args.table_name);

    if failed_uploads > 0 {
        println!("\n⚠️  Some files failed to upload. Check the error messages above for details.");
    }

    Ok(())
}

/// Create a database connection with proper timeout and error handling
async fn create_db_connection(postgres_url: &str) -> Result<Client> {
    println!("  🔌 Connecting to PostgreSQL...");

    // Add connection timeout
    let connect_future = tokio_postgres::connect(postgres_url, NoTls);
    let timeout_future = tokio::time::timeout(Duration::from_secs(15), connect_future);

    let (client, connection) = timeout_future
        .await
        .context("Database connection timed out after 15 seconds")?
        .context("Failed to connect to PostgreSQL")?;
    // Spawn the connection task with error handling
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("⚠️  Database connection error: {}", e);
        }
    });
    // Test the connection with timeout
    let test_future = client.simple_query("SELECT 1");
    tokio::time::timeout(Duration::from_secs(10), test_future)
        .await
        .context("Connection test timed out after 10 seconds")?
        .context("Failed to test database connection")?;

    println!("  ✅ Database connection established");

    Ok(client)
}

/// Load and validate all merkle tree JSON files from the folder
async fn load_and_validate_merkle_trees(
    args: &Args,
    upload_args: &UploadMerkleTreeArgs,
) -> Result<Vec<ValidatedMerkleTree>> {
    let mut paths: Vec<_> = fs::read_dir(&upload_args.merkle_tree_path)
        .context("Failed to read directory")?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            // Only process .json files
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    // Sort paths for consistent processing order
    paths.sort();

    if paths.is_empty() {
        return Err(anyhow::anyhow!(
            "No JSON files found in folder: {:?}",
            upload_args.merkle_tree_path
        ));
    }

    println!("Found {} JSON files to validate", paths.len());

    let mut validated_trees = Vec::new();
    let mut validation_errors = Vec::new();

    for (index, file_path) in paths.iter().enumerate() {
        println!(
            "Validating file {}/{}: {}",
            index + 1,
            paths.len(),
            file_path.file_name().unwrap().to_string_lossy()
        );

        match validate_single_merkle_tree(file_path, args) {
            Ok(validated_tree) => {
                println!(
                    "  ✅ Valid - Version: {}, Records: {}, PDA: {}",
                    validated_tree.merkle_tree.airdrop_version,
                    validated_tree.merkle_tree.tree_nodes.len(),
                    validated_tree.distributor_pda
                );
                validated_trees.push(validated_tree);
            }
            Err(e) => {
                let error_msg = format!(
                    "  ❌ Invalid - {}: {}",
                    file_path.file_name().unwrap().to_string_lossy(),
                    e
                );
                println!("{}", error_msg);
                validation_errors.push(error_msg);
            }
        }
    }
    if !validation_errors.is_empty() {
        println!("\n⚠️  Validation errors found:");
        for error in &validation_errors {
            println!("{}", error);
        }
        println!("Only valid files will be processed.\n");
    }

    // Sort by airdrop version for consistent processing order
    validated_trees.sort_by_key(|tree| tree.merkle_tree.airdrop_version);

    Ok(validated_trees)
}

/// Validate a single merkle tree JSON file
fn validate_single_merkle_tree(file_path: &PathBuf, args: &Args) -> Result<ValidatedMerkleTree> {
    // Load the merkle tree from JSON
    let merkle_tree = AirdropMerkleTree::new_from_file(file_path)
        .with_context(|| format!("Failed to parse JSON file: {:?}", file_path))?;

    // Validate basic structure
    if merkle_tree.tree_nodes.is_empty() {
        return Err(anyhow::anyhow!("Merkle tree has no nodes"));
    }
    if merkle_tree.max_num_nodes != merkle_tree.tree_nodes.len() as u64 {
        return Err(anyhow::anyhow!(
            "max_num_nodes ({}) doesn't match actual nodes count ({})",
            merkle_tree.max_num_nodes,
            merkle_tree.tree_nodes.len()
        ));
    }
    // Validate that all nodes have proofs
    for (index, node) in merkle_tree.tree_nodes.iter().enumerate() {
        if node.proof.is_none() {
            return Err(anyhow::anyhow!("Node at index {} is missing proof", index));
        }
    }

    // Generate and validate the distributor PDA
    println!("  🔍 PDA Calculation Debug:");
    println!("    Program ID: {}", args.program_id);
    println!("    Base Key: {}", args.base);
    println!("    Mint: {}", args.mint);
    println!("    Version: {}", merkle_tree.airdrop_version);

    let (distributor_pda, bump) = get_merkle_distributor_pda(
        &args.program_id,
        &args.base,
        &args.mint,
        merkle_tree.airdrop_version,
    );

    println!("    Calculated PDA: {}", distributor_pda);
    println!("    Bump: {}", bump);

    let distributor_index = bs58::encode(distributor_pda.to_bytes()).into_string();

    // Validate merkle root integrity (this also validates all proofs)
    merkle_tree
        .verify_proof()
        .with_context(|| "Merkle tree proof verification failed")?;

    Ok(ValidatedMerkleTree {
        merkle_tree,
        file_path: file_path.clone(),
        distributor_pda,
        distributor_index,
    })
}

/// Process a single validated merkle tree
async fn process_single_merkle_tree(
    client: &mut Client,
    validated_tree: &ValidatedMerkleTree,
    table_name: &str,
) -> Result<u64> {
    // Check if records already exist FIRST
    let existing_count: i64 = client
        .query_one(
            &format!("SELECT COUNT(*) FROM {} WHERE \"index\" = $1", table_name),
            &[&validated_tree.distributor_index],
        )
        .await
        .with_context(|| "Failed to check existing records")?
        .get(0);

    let expected_count = validated_tree.merkle_tree.tree_nodes.len() as i64;

    if existing_count >= expected_count {
        println!(
            "  ✅ All {} records already exist for this distributor - skipping upload",
            existing_count
        );
        return Ok(0); // Return 0 since no new records were uploaded
    }

    if existing_count > 0 {
        println!(
            "  ⚠️  Found {} existing records, need to upload {} more",
            existing_count,
            expected_count - existing_count
        );
    }

    let conversion_start = std::time::Instant::now();

    // Pre-compute all string conversions
    let prepared_records: Vec<PreparedRecord> = validated_tree
        .merkle_tree
        .tree_nodes
        .iter()
        .map(|tree_node| PreparedRecord {
            recipient_base58: bs58::encode(tree_node.claimant.to_bytes()).into_string(),
            amount_hex: format!("0x{:x}", tree_node.amount),
            proof_hex_array: if let Some(ref proof) = tree_node.proof {
                proof
                    .iter()
                    .map(|p| format!("0x{}", hex::encode(p)))
                    .collect()
            } else {
                Vec::new()
            },
        })
        .collect();

    println!(
        "  📝 Pre-computed {} records in {:.2?}",
        prepared_records.len(),
        conversion_start.elapsed()
    );

    // Upload the data
    let uploaded_count = upload_merkle_tree_data_optimized(
        client,
        &prepared_records,
        table_name,
        &validated_tree.distributor_index,
    )
    .await?;

    Ok(uploaded_count)
}

/// Optimized upload using bulk operations and prepared statements
async fn upload_merkle_tree_data_optimized(
    client: &mut Client,
    prepared_records: &[PreparedRecord],
    table_name: &str,
    distributor_index: &str,
) -> Result<u64> {
    // Use smaller batch sizes to avoid connection issues
    const BATCH_SIZE: usize = 500;
    let _total_records = prepared_records.len() as u64;

    let mut total_inserted = 0;

    // Process in smaller chunks to avoid timeouts
    let chunks: Vec<_> = prepared_records.chunks(BATCH_SIZE).collect();
    let total_batches = chunks.len();

    for (batch_idx, chunk) in chunks.iter().enumerate() {
        let batch_start = std::time::Instant::now();

        // Retry logic for each batch
        let mut retry_count = 0;
        const MAX_RETRIES: usize = 3;

        loop {
            // Add timeout to prevent hanging
            let batch_future = process_single_batch(
                client,
                chunk,
                table_name,
                distributor_index,
                batch_idx + 1,
                total_batches,
            );
            let timeout_future = tokio::time::timeout(Duration::from_secs(60), batch_future);

            match timeout_future.await {
                Ok(Ok(rows_affected)) => {
                    total_inserted += rows_affected;
                    println!(
                        "    Batch {}/{}: {} new records in {:.2?}",
                        batch_idx + 1,
                        total_batches,
                        rows_affected,
                        batch_start.elapsed()
                    );
                    break;
                }
                Ok(Err(e)) => {
                    retry_count += 1;
                    if retry_count >= MAX_RETRIES {
                        return Err(anyhow::anyhow!("Database error: {}", e));
                    }
                    println!(
                        "    ⚠️  Batch {}/{} failed (attempt {}), retrying in 2s: {}",
                        batch_idx + 1,
                        total_batches,
                        retry_count,
                        e
                    );
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
                Err(_) => {
                    retry_count += 1;
                    if retry_count >= MAX_RETRIES {
                        return Err(anyhow::anyhow!(
                            "Batch processing timed out after 60 seconds"
                        ));
                    }
                    println!(
                        "    ⏰ Batch {}/{} timed out (attempt {}), retrying in 5s...",
                        batch_idx + 1,
                        total_batches,
                        retry_count
                    );
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
        // Longer delay between batches to prevent overwhelming the database
        if batch_idx < chunks.len() - 1 {
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }
    Ok(total_inserted)
}

/// Process a single batch with error handling
async fn process_single_batch(
    client: &mut Client,
    chunk: &[PreparedRecord],
    table_name: &str,
    distributor_index: &str,
    _batch_num: usize,
    _total_batches: usize,
) -> Result<u64, PgError> {
    // Start transaction for this batch
    let transaction = client.transaction().await?;
    // Build bulk insert with VALUES clause
    let mut query = format!(
        r#"INSERT INTO {} ("index", recipient, amount, proof) VALUES"#,
        table_name
    );

    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
    let mut param_idx = 1;

    for (i, record) in chunk.iter().enumerate() {
        if i > 0 {
            query.push_str(", ");
        }
        query.push_str(&format!(
            " (${}, ${}, ${}, ${})",
            param_idx,
            param_idx + 1,
            param_idx + 2,
            param_idx + 3
        ));

        params.push(&distributor_index);
        params.push(&record.recipient_base58);
        params.push(&record.amount_hex);
        params.push(&record.proof_hex_array);

        param_idx += 4;
    }

    query.push_str(" ON CONFLICT DO NOTHING");

    // Execute bulk insert
    let rows_affected = transaction.execute(&query, &params).await?;
    transaction.commit().await?;

    Ok(rows_affected)
}

/// Synchronous wrapper for the async upload function
pub fn process_upload_merkle_tree_sync(
    args: &Args,
    upload_args: &UploadMerkleTreeArgs,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new().context("Failed to create Tokio runtime")?;

    rt.block_on(process_upload_merkle_tree(args, upload_args))
}
