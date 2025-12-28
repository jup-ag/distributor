#![allow(clippy::too_many_arguments)]

use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke_signed, system_instruction},
};
use anchor_spl::{associated_token::ID as ATA_ID, token};

use crate::{
    error::ErrorCode,
    state::pino_distributor::{
        md_init_zero, md_load_mut, ActivationType as PinoActivationType, PinoAirdropBonus,
        PinoMerkleDistributor,
    },
};

#[cfg(feature = "localnet")]
const SECONDS_PER_DAY: i64 = 0;
#[cfg(not(feature = "localnet"))]
const SECONDS_PER_DAY: i64 = 24 * 3600;

#[inline(always)]
fn parse_new_distributor_args(ix_data: &[u8]) -> Result<(
    u64,          // version
    [u8; 32],     // root
    u64,          // max_total_claim
    u64,          // max_num_nodes
    i64,          // start_vesting_ts
    i64,          // end_vesting_ts
    i64,          // clawback_start_ts
    u64,          // activation_point
    u8,           // activation_type
    bool          // closable
)> {
    // Anchor layout: [8 discriminator] + fields (borsh)
    if ix_data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData.into());
    }
    let mut cur = &ix_data[8..];

    fn take<const N: usize>(cur: &mut &[u8]) -> Result<[u8; N]> {
        if cur.len() < N {
            return Err(ProgramError::InvalidInstructionData.into());
        }
        let (a, b) = cur.split_at(N);
        *cur = b;
        Ok(a.try_into().unwrap())
    }

    let version        = u64::from_le_bytes(take::<8>(&mut cur)?);
    let root           = take::<32>(&mut cur)?;
    let max_total      = u64::from_le_bytes(take::<8>(&mut cur)?);
    let max_nodes      = u64::from_le_bytes(take::<8>(&mut cur)?);
    let start_ts       = i64::from_le_bytes(take::<8>(&mut cur)?);
    let end_ts         = i64::from_le_bytes(take::<8>(&mut cur)?);
    let clawback_start = i64::from_le_bytes(take::<8>(&mut cur)?);
    let activation_pt  = u64::from_le_bytes(take::<8>(&mut cur)?);
    let activation_ty  = take::<1>(&mut cur)?[0];
    let closable       = match take::<1>(&mut cur)?[0] {
        0 => false,
        1 => true,
        _ => return Err(ProgramError::InvalidInstructionData.into()),
    };

    Ok((
        version,
        root,
        max_total,
        max_nodes,
        start_ts,
        end_ts,
        clawback_start,
        activation_pt,
        activation_ty,
        closable,
    ))
}

/// Fast-path initializer: creates & populates a PinoMerkleDistributor PDA.
/// Expected accounts
/// 0. [writable] distributor PDA
/// 1. [signer]   base
/// 2. [writable] clawback_receiver (SPL Token account)
/// 3. []         mint (SPL Mint)
/// 4. [writable] token_vault (ATA of distributor, classic token program)
/// 5. [signer]   admin (payer)
/// 6. []         system_program
/// 7. []         associated_token_program
/// 8. []         token_program
pub fn p_handle_new_distributor<'info>(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'info>],
    ix_data: &[u8],
) -> Result<()> {
    let [
    distributor_ai,
    base_ai,
    clawback_receiver_ai,
    mint_ai,
    token_vault_ai,
    admin_ai,
    system_program_ai,
    associated_token_program_ai,
    token_program_ai,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    // Basic program ID 
    if system_program_ai.key() != System::id() {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    if associated_token_program_ai.key() != ATA_ID {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    if token_program_ai.key() != token::ID {
        return Err(ProgramError::IncorrectProgramId.into());
    }

    if !base_ai.is_signer {
        return Err(ProgramError::MissingRequiredSignature.into());
    }
    if !admin_ai.is_signer {
        return Err(ProgramError::MissingRequiredSignature.into());
    }

    // Parse args 
    let (
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
    ) = parse_new_distributor_args(ix_data)?;

    // Derive distributor PDA + bump from seeds 
    let (expected_distributor, bump) = Pubkey::find_program_address(
        &[
            b"MerkleDistributor",
            base_ai.key.as_ref(),
            mint_ai.key.as_ref(),
            &version.to_le_bytes(),
        ],
        program_id,
    );
    if distributor_ai.key() != expected_distributor {
        return Err(ProgramError::InvalidSeeds.into());
    }

    // Validate token vault owner/mint consistency
    if token_vault_ai.owner != token_program_ai.key {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    let vault_data = token_vault_ai.try_borrow_data()?;
    if vault_data.len() < 64 {
        return Err(ProgramError::InvalidAccountData.into());
    }

    let mut mint_bytes = [0u8; 32];
    let mut owner_bytes = [0u8; 32];
    mint_bytes.copy_from_slice(&vault_data[0..32]);
    owner_bytes.copy_from_slice(&vault_data[32..64]);
    drop(vault_data);

    if Pubkey::new_from_array(mint_bytes) != *mint_ai.key {
        return Err(ProgramError::InvalidAccountData.into());
    }
    if Pubkey::new_from_array(owner_bytes) != *distributor_ai.key {
        return Err(ProgramError::InvalidAccountData.into());
    }
    if clawback_receiver_ai.owner != token_program_ai.key {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    let clawback_data = clawback_receiver_ai.try_borrow_data()?;
    if clawback_data.len() < 64 {
        return Err(ProgramError::InvalidAccountData.into());
    }
    let mut clawback_mint_bytes = [0u8; 32];
    clawback_mint_bytes.copy_from_slice(&clawback_data[0..32]);
    drop(clawback_data);
    if Pubkey::new_from_array(clawback_mint_bytes) != *mint_ai.key {
        return Err(ProgramError::InvalidAccountData.into());
    }

    // Timestamp checks
    let curr_ts = Clock::get()?.unix_timestamp;
    if start_vesting_ts >= end_vesting_ts {
        return Err(ErrorCode::StartTimestampAfterEnd.into());
    }
    if !(start_vesting_ts > curr_ts && end_vesting_ts > curr_ts && clawback_start_ts > curr_ts) {
        return Err(ErrorCode::TimestampsNotInFuture.into());
    }
    if clawback_start_ts <= end_vesting_ts {
        return Err(ErrorCode::ClawbackDuringVesting.into());
    }
    if clawback_start_ts
        < end_vesting_ts
        .checked_add(SECONDS_PER_DAY)
        .ok_or(ErrorCode::ArithmeticError)?
    {
        return Err(ErrorCode::InsufficientClawbackDelay.into());
    }
    PinoActivationType::try_from(activation_type)
        .map_err(|_| ErrorCode::InvalidActivationType)?;

    if distributor_ai.lamports() > 0 && distributor_ai.owner == program_id {
        return Err(ProgramError::AccountAlreadyInitialized.into());
    }
    
    // Create the distributor account if needed (PDA must "sign")
    if distributor_ai.lamports() == 0 || distributor_ai.owner != program_id {
        let lamports = Rent::get()?.minimum_balance(PinoMerkleDistributor::LEN);
        let create_ix = system_instruction::create_account(
            admin_ai.key,
            &expected_distributor,
            lamports,
            PinoMerkleDistributor::LEN as u64,
            program_id,
        );

        let seeds: [&[u8]; 5] = [
            b"MerkleDistributor",
            base_ai.key.as_ref(),
            mint_ai.key.as_ref(),
            &version.to_le_bytes(),
            &[bump],
        ];
        invoke_signed(
            &create_ix,
            &[
                admin_ai.clone(),
                distributor_ai.clone(),
                system_program_ai.clone(),
            ],
            &[&seeds],
        )?;
    }
    if distributor_ai.owner != program_id {
        return Err(ProgramError::IncorrectProgramId.into());
    }

    // Initialize PinoMerkleDistributor in place
    md_init_zero(distributor_ai)?;
    {
        let d = md_load_mut(distributor_ai)?;
        d.bump = bump;
        d.version = version;
        d.root = root;
        d.mint = *mint_ai.key;
        d.base = *base_ai.key;
        d.token_vault = *token_vault_ai.key;
        d.max_total_claim = max_total_claim;
        d.max_num_nodes = max_num_nodes;
        d.total_amount_claimed = 0;
        d.num_nodes_claimed = 0;
        d.start_ts = start_vesting_ts;
        d.end_ts = end_vesting_ts;
        d.clawback_start_ts = clawback_start_ts;
        d.clawback_receiver = *clawback_receiver_ai.key;
        d.admin = *admin_ai.key;
        d.set_clawed_back(false);
        d.activation_point = activation_point;
        d.activation_type = activation_type;
        d.set_closable(closable);
        d.airdrop_bonus = PinoAirdropBonus::default();
    }

    msg!(
        "New distributor (fast) v={} mint={} vault={} max_total={} max_nodes={} start={} end={} clawback={} activation_point={} activation_type={} closable={}",
        version,
        mint_ai.key,
        token_vault_ai.key,
        max_total_claim,
        max_num_nodes,
        start_vesting_ts,
        end_vesting_ts,
        clawback_start_ts,
        activation_point,
        activation_type,
        closable,
    );

    Ok(())
}
