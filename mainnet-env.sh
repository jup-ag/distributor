#!/bin/bash
# WCT Token Distribution - Mainnet Environment Configuration
# IMPORTANT: Update the placeholder values before deployment!

# Token Configuration
export TOKEN_MINT="WCTk5xWdn5SYg56twGj32sUF3W4WFQ48ogezLBuYTBY"
export TOKEN_DECIMALS="9"

# Key Paths
export BASE_PATH="keys/base-mainnet.json"
export KEYPAIR_PATH="keys/admin-mainnet.json"  # Your funded keypair
export BASE_KEY=$(solana-keygen pubkey $BASE_PATH 2>/dev/null || echo "BASE_KEY_NOT_GENERATED")

# RPC Configuration
# Replace with your QuickNode Blockchain API URL
export RPC="YOUR-QUICKNODE-URL"

# File Paths
export CSV_PATH="./your_recipients.csv"  # Update with your final CSV filename
export MERKLE_TREE_PATH="./merkle_trees_mainnet"

# Admin Configuration
export ADMIN=$(solana-keygen pubkey $KEYPAIR_PATH 2>/dev/null || echo "ADMIN_KEY_NOT_FOUND")
export ADMIN_MULTISIG="5cbceZGXaUGxkJcscqhWw4QTMjx2ZTi6xo9TXTnp8v6i"  # 2/6 Squads
export TREASURY_MULTISIG="FGfjkeAvmodQFzWBko3fShcL8XM3APRG8z5JWtkfBev1"  # 2/3 Squads
export CLAWBACK_RECEIVER_OWNER=$TREASURY_MULTISIG

# Claim Configuration (Permissionless)
export CLAIM_TYPE=0  # 0 = Permissionless
export OPERATOR="11111111111111111111111111111111"  # System program (not needed)
export LOCKER="11111111111111111111111111111111"    # System program (not needed)

# Timing Configuration
# For QA deployment with claims starting in 5 minutes
export CLAIM_START_DATE=$(date -u -d "+5 minutes" +"%Y-%m-%d %H:%M:%S UTC")  # 5 minutes from now
export CLAWBACK_DATE="2025-08-14 00:00:00 UTC"  # 1 month from July 14th 2025

# Priority Fee (microlamports)
# Check https://www.quicknode.com/gas-tracker/solana for current rates
# Common values: 1000 (low), 50000 (medium), 1000000 (high)
export PRIORITY_FEE="50000"

# Program ID (Jupiter's merkle distributor deployment)
export PROGRAM_ID="DiS3nNjFVMieMgmiQFm6wgJL7nevk4NrhXKLbtEH1Z2R"

# Bonus parameters (not used for simple distributions)
export BONUS_VESTING_DURATION=0
export BONUS_MULTIPLIER=0

# Distributor configuration
export CLOSABLE="true"  # CRITICAL: Allows immediate cancellation if critical errors found during QA

export POSTGRES_URL="SUPABASE_URL"

# Function to calculate timestamps when dates are set
calculate_timestamps() {
    if [[ "$CLAIM_START_DATE" == *"XX"* ]]; then
        echo "❌ ERROR: Please update CLAIM_START_DATE with actual date"
        return 1
    fi
    
    if [[ "$CLAWBACK_DATE" == *"XX"* ]]; then
        echo "❌ ERROR: Please update CLAWBACK_DATE with actual date"
        return 1
    fi
    
    # Calculate Unix timestamps
    export CLAIM_START_TS=$(date -d "$CLAIM_START_DATE" +%s 2>/dev/null || date -j -f "%Y-%m-%d %H:%M:%S %Z" "$CLAIM_START_DATE" +%s)
    # IMPORTANT: Following devnet guide timing for minimal vesting
    # Even "immediate" claims need 2-minute vesting period
    export START_VESTING_TS=$CLAIM_START_TS               # Vesting starts when claims start
    export END_VESTING_TS=$((START_VESTING_TS + 120))     # 2 minutes later (minimal vesting)
    export CLAWBACK_START_TS=$(date -d "$CLAWBACK_DATE" +%s 2>/dev/null || date -j -f "%Y-%m-%d %H:%M:%S %Z" "$CLAWBACK_DATE" +%s)
    
    # Use timestamp-based activation
    export ACTIVATION_POINT=$CLAIM_START_TS
    export ACTIVATION_TYPE=1  # 1 = timestamp-based activation
    
    echo "✅ Timestamps calculated:"
    echo "   Claim starts: $(date -d @$CLAIM_START_TS 2>/dev/null || date -r $CLAIM_START_TS)"
    echo "   Vesting ends: $(date -d @$END_VESTING_TS 2>/dev/null || date -r $END_VESTING_TS)"
    echo "   Clawback starts: $(date -d @$CLAWBACK_START_TS 2>/dev/null || date -r $CLAWBACK_START_TS)"
}

# Function to validate environment
validate_env() {
    echo "🔍 Validating environment configuration..."
    
    local errors=0
    
    # Check if placeholder values are updated
    if [[ "$RPC" == *"YOUR-QUICKNODE-URL"* ]]; then
        echo "❌ ERROR: Update RPC with your QuickNode URL"
        ((errors++))
    fi
    
    if [[ "$CSV_PATH" == "./your_recipients.csv" ]]; then
        echo "⚠️  WARNING: Update CSV_PATH with your actual recipients file"
    fi
    
    if [[ "$CLAIM_START_DATE" == *"XX"* ]]; then
        echo "❌ ERROR: Update CLAIM_START_DATE with actual date"
        ((errors++))
    fi
    
    if [[ "$CLAWBACK_DATE" == *"XX"* ]]; then
        echo "❌ ERROR: Update CLAWBACK_DATE with actual date"
        ((errors++))
    fi
    
    # Check if keys exist
    if [[ ! -f "$BASE_PATH" ]]; then
        echo "⚠️  WARNING: BASE_PATH key not found. Run: solana-keygen new --outfile $BASE_PATH"
    fi
    
    if [[ ! -f "$KEYPAIR_PATH" ]]; then
        echo "❌ ERROR: KEYPAIR_PATH not found at $KEYPAIR_PATH"
        ((errors++))
    fi
    
    if [[ ! -f "$CSV_PATH" ]]; then
        echo "❌ ERROR: CSV file not found at $CSV_PATH"
        ((errors++))
    fi
    
    if [[ $errors -gt 0 ]]; then
        echo "❌ Found $errors errors. Please fix them before proceeding."
        return 1
    else
        echo "✅ Environment configuration looks good!"
        return 0
    fi
}

# Print configuration summary
print_config() {
    echo "📋 WCT Distribution Configuration Summary"
    echo "========================================"
    echo "Token: $TOKEN_MINT (${TOKEN_DECIMALS} decimals)"
    echo "Admin: $ADMIN"
    echo "Admin Multisig: $ADMIN_MULTISIG"
    echo "Treasury: $TREASURY_MULTISIG"
    echo "Base Key: $BASE_KEY"
    echo "CSV Path: $CSV_PATH"
    echo "Claim Type: Permissionless"
    echo "Priority Fee: $PRIORITY_FEE microlamports"
    echo ""
    echo "Timing:"
    echo "  Claim Start: $CLAIM_START_DATE"
    echo "  Clawback: $CLAWBACK_DATE"
    echo "========================================"
}

# Auto-run validation when sourced
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    print_config
    validate_env
    if [[ $? -eq 0 ]]; then
        calculate_timestamps
    fi
else
    echo "✅ Environment loaded. Run 'validate_env' to check configuration."
    echo "   Run 'calculate_timestamps' when dates are set."
    echo "   Run 'print_config' to see current configuration."
fi