# WCT Token Distribution: Mainnet Production Deployment Guide

## Overview

This guide walks through deploying a production token distribution for WCT tokens on Solana mainnet using the Jupiter Merkle Distributor program.

### QA Strategy
- Deploy immediately with claims starting now
- Upload merkle proofs to private PostgreSQL database
- Password-protected frontend for controlled access
- Internal team and test wallets claim first
- Monitor for issues during QA phase
- Once QA complete, remove password and announce publicly

### Key Parameters

- **Token**: WCTk5xWdn5SYg56twGj32sUF3W4WFQ48ogezLBuYTBY (9 decimals)
- **Admin Multisig**: 5cbceZGXaUGxkJcscqhWw4QTMjx2ZTi6xo9TXTnp8v6i (2/6)
- **Treasury Multisig**: FGfjkeAvmodQFzWBko3fShcL8XM3APRG8z5JWtkfBev1 (2/3)
- **Distribution Type**: Permissionless claiming with merkle proofs
- **Program**: DiS3nNjFVMieMgmiQFm6wgJL7nevk4NrhXKLbtEH1Z2R (Jupiter's deployment)

## Prerequisites

### System Requirements

- **Rust**: 1.68.0+
- **Solana CLI**: 1.16.25+
- **Anchor**: 0.28.0+
- **PostgreSQL**: For merkle proof storage
- **QuickNode RPC**: Production RPC access

### Initial Setup

```bash
# Verify installations
solana --version
anchor --version
cargo --version

# Configure Solana for mainnet
solana config set --url https://api.mainnet-beta.solana.com

# Build the project
anchor build -p merkle_distributor
cargo build -p jup-scripts --release
```

## Phase 1: Key Management & Test Wallets

### 1. Create Directory Structure

```bash
# Create keys directory (ensure it's gitignored!)
mkdir -p keys
echo "*" > keys/.gitignore
echo "!.gitignore" >> keys/.gitignore
```

### 2. Generate BASE_KEY

```bash
# Generate unique base key for your distributor
solana-keygen new --outfile keys/base-mainnet.json

# Save the pubkey
export BASE_KEY=$(solana-keygen pubkey keys/base-mainnet.json)
echo "BASE_KEY: $BASE_KEY"
```

### 3. Generate Test Wallets

```bash
# Build and run the test wallet generator
cargo build --release --bin generate_test_wallets

./target/release/generate_test_wallets \
  --count 200 \
  --min-amount 0.01 \
  --max-amount 0.01 \
  --output test_wallets_CONFIDENTIAL.json \
  --csv test_recipients.csv

# IMPORTANT: Secure the private keys file!
chmod 600 test_wallets_CONFIDENTIAL.json
```

### 4. Merge Test Recipients with Your CSV

```bash
# Backup your original CSV
cp your_recipients.csv your_recipients_backup.csv

# Append test recipients (skip header)
tail -n +2 test_recipients.csv >> your_recipients.csv

# Verify the merge
echo "Total recipients: $(wc -l < your_recipients.csv)"
head -5 your_recipients.csv
tail -5 your_recipients.csv
```

## Phase 2: Environment Configuration

### 5. Set Production Variables

Create a file `mainnet-env.sh`:

```bash
#!/bin/bash

# Token Configuration
export TOKEN_MINT="WCTk5xWdn5SYg56twGj32sUF3W4WFQ48ogezLBuYTBY"
export TOKEN_DECIMALS="9"

# Key Paths
export BASE_PATH="keys/base-mainnet.json"
export KEYPAIR_PATH="keys/admin-mainnet.json"  # Your funded keypair
export BASE_KEY=$(solana-keygen pubkey $BASE_PATH)

# RPC Configuration
export RPC="YOUR_QUICKNODE_MAINNET_URL"  # Replace with your QuickNode URL

# File Paths
export CSV_PATH="./your_recipients.csv"  # Your final CSV with test wallets
export MERKLE_TREE_PATH="./merkle_trees_mainnet"

# Admin Configuration
export ADMIN=$(solana-keygen pubkey $KEYPAIR_PATH)
export ADMIN_MULTISIG="5cbceZGXaUGxkJcscqhWw4QTMjx2ZTi6xo9TXTnp8v6i"
export TREASURY_MULTISIG="FGfjkeAvmodQFzWBko3fShcL8XM3APRG8z5JWtkfBev1"

# Claim Configuration (Permissionless)
export CLAIM_TYPE=0
export OPERATOR="11111111111111111111111111111111"
export LOCKER="11111111111111111111111111111111"

# Timing Placeholders (UPDATE THESE WHEN READY!)
export CLAIM_START_DATE="2024-XX-XX 00:00:00 UTC"  # UPDATE ME
export CLAWBACK_DATE="2024-XX-XX 00:00:00 UTC"    # UPDATE ME

# Priority Fee (check https://www.quicknode.com/gas-tracker/solana)
export PRIORITY_FEE="26342"  # Adjust based on network conditions

# Program ID (Jupiter's deployment)
export PROGRAM_ID="DiS3nNjFVMieMgmiQFm6wgJL7nevk4NrhXKLbtEH1Z2R"
```

### 6. Calculate Timing Parameters

```bash
# Source the environment
source mainnet-env.sh

# When you're ready to deploy, update these:
# Convert your dates to Unix timestamps
export CLAIM_START_TS=$(date -d "$CLAIM_START_DATE" +%s)
export START_VESTING_TS=$((CLAIM_START_TS + 60))      # 1 minute after claim start
export END_VESTING_TS=$((START_VESTING_TS + 60))      # 2 minutes total (minimal vesting)
export CLAWBACK_START_TS=$(date -d "$CLAWBACK_DATE" +%s)

# Get activation slot
export ACTIVATION_POINT=$CLAIM_START_TS
export ACTIVATION_TYPE=1  # 1 = timestamp-based activation

echo "Claim starts: $(date -d @$CLAIM_START_TS)"
echo "Vesting ends: $(date -d @$END_VESTING_TS)"
echo "Clawback starts: $(date -d @$CLAWBACK_START_TS)"
```

## Phase 3: Merkle Tree Generation

### 7. Validate CSV Data

```bash
# Check CSV format
head -10 $CSV_PATH

# Validate structure (should have 3 columns: pubkey,amount,locked_amount)
awk -F',' 'NR==1{if(NF!=3 || $1!="pubkey" || $2!="amount" || $3!="locked_amount") exit 1}' $CSV_PATH
if [ $? -eq 0 ]; then echo "CSV header valid"; else echo "CSV header invalid!"; fi

# Count total tokens to distribute
awk -F',' 'NR>1{sum+=$2}END{print "Total tokens to distribute: " sum}' $CSV_PATH
```

### 8. Generate Merkle Trees

```bash
# Create merkle tree files
./target/release/cli create-merkle-tree \
  --csv-path $CSV_PATH \
  --merkle-tree-path $MERKLE_TREE_PATH \
  --max-nodes-per-tree 10000 \
  --amount 1000 \
  --decimals $TOKEN_DECIMALS

# Verify generation
ls -la $MERKLE_TREE_PATH/
echo "Number of trees: $(ls $MERKLE_TREE_PATH/*.json | wc -l)"
```

## Phase 4: Deployment

### 9. Pre-deployment Checklist

```bash
# Check admin wallet SOL balance (only needs SOL for fees)
echo "Admin SOL balance: $(solana balance $ADMIN --url $RPC)"

# Check treasury token balance (tokens will come from here via Squads)
echo "Treasury token balance: $(spl-token balance $TOKEN_MINT --owner $TREASURY_MULTISIG --url $RPC)"

# Calculate required tokens
REQUIRED_TOKENS=$(awk -F',' 'NR>1{sum+=$2}END{print sum}' $CSV_PATH)
echo "Required tokens: $REQUIRED_TOKENS"
echo "Note: Tokens will be sent from Treasury Squads, not admin wallet"
```

### 10. Deploy Distributor

```bash
# TIMING REQUIREMENTS (from devnet guide):
# 1. Claims must start in the future (5 minutes buffer)
# 2. Even "immediate" claims require 2-minute vesting period
# 3. Tokens are claimable at CLAIM_START but fully vested at END_VESTING
# Deploy quickly after sourcing the environment!

# Deploy the merkle distributor
./target/release/cli \
  --mint $TOKEN_MINT \
  --priority-fee $PRIORITY_FEE \
  --keypair-path $KEYPAIR_PATH \
  --rpc-url $RPC \
  --program-id $PROGRAM_ID \
  new-distributor \
  --start-vesting-ts $START_VESTING_TS \
  --end-vesting-ts $END_VESTING_TS \
  --merkle-tree-path $MERKLE_TREE_PATH \
  --base-path $BASE_PATH \
  --clawback-start-ts $CLAWBACK_START_TS \
  --activation-point $ACTIVATION_POINT \
  --activation-type $ACTIVATION_TYPE \
  --clawback-receiver-owner $TREASURY_MULTISIG \
  --closable \
  --bonus-vesting-duration 0 \
  --bonus-multiplier 0 \
  --operator $OPERATOR \
  --locker $LOCKER \
  --claim-type $CLAIM_TYPE

# Save the output! It will show distributor addresses
```

### 11. Fund Distributor from Treasury

```bash
# Get the distributor addresses and required amounts
./target/release/cli \
  --mint $TOKEN_MINT \
  --base $BASE_KEY \
  --rpc-url $RPC \
  --program-id $PROGRAM_ID \
  view-distributors \
  --from-version 0 \
  --to-version 10

# Note the distributor addresses and token vault addresses from the output above

# IMPORTANT: Send tokens from Treasury Squads (2/3 multisig)
# 1. Go to your Squads UI at https://v3.squads.so/
# 2. Connect to your Treasury vault (FGfjkeAvmodQFzWBko3fShcL8XM3APRG8z5JWtkfBev1)
# 3. For each distributor, create a token transfer transaction:
#    - Token: WCT
#    - Recipient: [Distributor's token vault address]
#    - Amount: [Max total claim amount shown in view-distributors]
# 4. Have 2 of 3 signers approve and execute the transaction

# Alternative: If you want to fund from admin wallet instead:
# First transfer tokens from Treasury to admin wallet via Squads, then:
# ./target/release/cli \
#   --mint $TOKEN_MINT \
#   --base $BASE_KEY \
#   --keypair-path $KEYPAIR_PATH \
#   --rpc-url $RPC \
#   --program-id $PROGRAM_ID \
#   fund-all \
#   --merkle-tree-path $MERKLE_TREE_PATH

# Verify funding completed
./target/release/cli \
  --mint $TOKEN_MINT \
  --base $BASE_KEY \
  --rpc-url $RPC \
  --program-id $PROGRAM_ID \
  view-distributors \
  --from-version 0 \
  --to-version 10 | grep -E "(version|amount|token_vault)"
```

## Phase 5: Database Upload

### 12. Upload Merkle Proofs to PostgreSQL

```bash
# Upload immediately to test full flow (API is private/password-protected)

```bash
# Set database connection
export POSTGRES_URL="host=YOUR_HOST user=YOUR_USER dbname=YOUR_DB password=YOUR_PASSWORD"

# Upload merkle tree data
./target/release/cli \
  --program-id $PROGRAM_ID \
  --base $BASE_KEY \
  --mint $TOKEN_MINT \
  upload-merkle-tree \
  --postgres-url "$POSTGRES_URL" \
  --merkle-tree-path $MERKLE_TREE_PATH \
  --table-name "airdrop_recipients"

# Verify upload
psql "$POSTGRES_URL" -c "SELECT COUNT(*) FROM airdrop_recipients;"
```

## Phase 6: Transfer Admin Control

### 13. Transfer Admin to Multisig

```bash
# CRITICAL: Transfer admin control to your Squads multisig
./target/release/cli \
  --mint $TOKEN_MINT \
  --base $BASE_KEY \
  --keypair-path $KEYPAIR_PATH \
  --rpc-url $RPC \
  --program-id $PROGRAM_ID \
  set-admin \
  --new-admin $ADMIN_MULTISIG \
  --merkle-tree-path $MERKLE_TREE_PATH

# Verify the transfer
./target/release/cli \
  --mint $TOKEN_MINT \
  --base $BASE_KEY \
  --rpc-url $RPC \
  --program-id $PROGRAM_ID \
  view-distributors \
  --from-version 0 \
  --to-version 10 | grep -i admin
```

## Phase 7: Internal QA Phase

### QA Deployment

```bash
# Source environment (claims start immediately)
source mainnet-env.sh

# Verify QA timing
echo "QA claims start: $CLAIM_START_DATE"
echo "Clawback date: $CLAWBACK_DATE"
```

### 14. Test Claims with Generated Wallets

```bash
# Test claim with first test wallet
# Extract test wallet info from test_wallets_CONFIDENTIAL.json
TEST_WALLET_1=$(jq -r '.wallets[0].pubkey' test_wallets_CONFIDENTIAL.json)
TEST_KEYPAIR_1=$(jq -r '.wallets[0].keypair' test_wallets_CONFIDENTIAL.json)

# Save test keypair temporarily
echo "$TEST_KEYPAIR_1" > /tmp/test_wallet_1.json

# Fund test wallet with SOL for transaction fees
solana transfer $TEST_WALLET_1 0.01 --keypair-path $KEYPAIR_PATH --url $RPC

# Test claim
./target/release/cli \
  --mint $TOKEN_MINT \
  --base $BASE_KEY \
  --keypair-path /tmp/test_wallet_1.json \
  --rpc-url $RPC \
  --program-id $PROGRAM_ID \
  claim \
  --merkle-tree-path $MERKLE_TREE_PATH

# Check balance
spl-token balance $TOKEN_MINT --owner $TEST_WALLET_1 --url $RPC

# Clean up
rm /tmp/test_wallet_1.json
```

## Phase 8: Production Launch

### 15. Final Checklist

- [ ] All test claims working correctly
- [ ] Admin transferred to multisig
- [ ] Database populated with merkle proofs
- [ ] API endpoints tested (if using custom API)
- [ ] Claim start time confirmed
- [ ] Clawback date confirmed
- [ ] Treasury multisig verified
- [ ] Priority fees optimized for current network
- [ ] Monitoring in place

### 16. Emergency Procedures

#### Immediate Cancellation (QA Phase Only)

```bash
# ONLY works if:
# 1. Distributor was created with --closable
# 2. Admin hasn't been transferred to multisig yet

# Close distributor and recover ALL tokens
./target/release/cli \
  --mint $TOKEN_MINT \
  --base $BASE_KEY \
  --keypair-path $KEYPAIR_PATH \
  --rpc-url $RPC \
  --program-id $PROGRAM_ID \
  close-distributor \
  --merkle-tree-path $MERKLE_TREE_PATH \
  --destination-owner $TREASURY_MULTISIG

echo "Distributor closed, all tokens returned to treasury"
```

#### Standard Procedures

```bash
# View distributor status (can be done by anyone)
./target/release/cli \
  --mint $TOKEN_MINT \
  --base $BASE_KEY \
  --rpc-url $RPC \
  --program-id $PROGRAM_ID \
  view-distributors \
  --from-version 0 \
  --to-version 10

# Clawback unclaimed tokens (after clawback date)
# Requires multisig after admin transfer
# This must be done through Squads UI
```

## Cost Estimates

### Mainnet Deployment Costs:

- **Distributor Creation**: ~0.5-1 SOL per tree
- **Token Account Rent**: ~0.002 SOL per distributor
- **Transaction Fees**: ~0.001-0.01 SOL per transaction
- **Priority Fees**: Variable based on network
- **Total Estimate**: ~2-5 SOL for deployment (SOL only)
- **Token Funding**: Done separately via Treasury Squads

## Security Reminders

1. **NEVER share `test_wallets_CONFIDENTIAL.json`**
2. **Store BASE_KEY securely** - it identifies your distributor
3. **Admin control is transferred to multisig** - further changes require Squads
4. **Test thoroughly** with generated wallets before announcing
5. **Monitor claims** to ensure smooth distribution

## Support Scripts

### Quick Status Check

```bash
# Save as check_status.sh
#!/bin/bash
source mainnet-env.sh

echo "=== Distributor Status ==="
./target/release/cli \
  --mint $TOKEN_MINT \
  --base $BASE_KEY \
  --rpc-url $RPC \
  --program-id $PROGRAM_ID \
  view-distributors \
  --from-version 0 \
  --to-version 10
```

### Database Query Helper

```sql
-- Check claim status for a specific wallet
SELECT * FROM airdrop_recipients WHERE recipient = 'WALLET_ADDRESS_HERE';

-- Count total claims possible
SELECT COUNT(DISTINCT recipient) as unique_recipients,
       SUM(CAST(amount AS DECIMAL)) as total_tokens
FROM airdrop_recipients;

-- Find test wallets
SELECT * FROM airdrop_recipients
WHERE amount IN ('0x2386f26fc10000', '0x6f05b59d3b20000', '0xa688906bd8b0000')
LIMIT 10;
```

## Recommended QA Workflow

### Phase 1: QA Deployment
1. Deploy with immediate claim start (NOW)
2. Fund from Treasury via Squads
3. Upload merkle proofs to PostgreSQL (private API)
4. **DO NOT transfer admin to multisig yet**

### Phase 2: Internal QA
1. Access password-protected claim frontend
2. Claim with test wallets (50 generated wallets)
3. Have team members claim their allocations
4. Test complete flow including:
   - API merkle proof retrieval
   - Frontend claim process
   - Transaction success
5. Monitor for any issues:
   - Claim failures
   - Incorrect amounts
   - UI/UX problems
   - API performance

### Phase 3: Issue Resolution
If critical errors found:
1. Use `close-distributor` to cancel (recovers ALL tokens)
2. Fix issues
3. Generate new BASE_KEY and redeploy

If minor issues:
1. Document and fix for next season
2. Continue to Phase 4

### Phase 4: Public Launch
1. Remove password protection from frontend
2. Transfer admin to Squads multisig
3. Update claim UI/documentation
4. Public announcement
5. Monitor public claims

## Next Steps

1. Deploy immediately for QA
2. Internal team claims first
3. 1-2 week QA period
4. Public launch when confident
5. 30-day claim window from public announcement

Remember: Once admin is transferred to the multisig, all admin operations (like clawback) must go through Squads!
