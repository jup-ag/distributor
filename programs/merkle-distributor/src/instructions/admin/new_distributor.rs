use crate::error::ErrorCode::ArithmeticError;
use crate::state::distributor_root::DistributorRoot;
use crate::state::merkle_distributor::{ActivationType, ClaimType};
use crate::{
    error::ErrorCode,
    state::merkle_distributor::{AirdropBonus, MerkleDistributor},
};
use anchor_lang::{account, context::Context, prelude::*, Accounts, Key, ToAccountInfo};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[cfg(feature = "localnet")]
const SECONDS_PER_DAY: i64 = 0;

#[cfg(not(feature = "localnet"))]
const SECONDS_PER_DAY: i64 = 24 * 3600; // 24 hours * 3600 seconds

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct NewDistributorParams {
    pub version: u64,
    pub total_claim: u64,
    pub max_num_nodes: u64,
    pub start_vesting_ts: i64,
    pub end_vesting_ts: i64,
    pub clawback_start_ts: i64,
    pub activation_point: u64, // can be slot or timestamp
    pub activation_type: u8,
    pub closable: bool,
    pub total_bonus: u64,
    pub bonus_vesting_duration: u64,
    pub claim_type: u8,
    pub operator: Pubkey,
    pub locker: Pubkey
}

impl NewDistributorParams {
    pub fn get_max_total_claim(&self) -> Result<u64> {
        let max_total_claim = self
            .total_claim
            .checked_add(self.total_bonus)
            .ok_or(ArithmeticError)?;
        Ok(max_total_claim)
    }
    fn get_airdrop_bonus(&self) -> AirdropBonus {
        AirdropBonus {
            total_bonus: self.total_bonus,
            vesting_duration: self.bonus_vesting_duration,
            total_claimed_bonus: 0,
        }
    }

    pub fn validate(&self) -> Result<()> {
        ActivationType::try_from(self.activation_type)
            .map_err(|_| ErrorCode::InvalidActivationType)?;

        let curr_ts = Clock::get()?.unix_timestamp;

        require!(
            self.start_vesting_ts < self.end_vesting_ts,
            ErrorCode::StartTimestampAfterEnd
        );

        require!(
            self.clawback_start_ts > self.end_vesting_ts,
            ErrorCode::ClawbackDuringVesting
        );

        // New distributor parameters must all be set in the future
        require!(
            self.start_vesting_ts > curr_ts,
            ErrorCode::TimestampsNotInFuture
        );

        // Ensure clawback_start_ts is at least one day after end_vesting_ts
        require!(
            self.clawback_start_ts
                >= self
                    .end_vesting_ts
                    .checked_add(SECONDS_PER_DAY)
                    .ok_or(ErrorCode::ArithmeticError)?,
            ErrorCode::InsufficientClawbackDelay
        );

        // validate claim type
        let claim_type_enum =
            ClaimType::try_from(self.claim_type).map_err(|_| ErrorCode::TypeCastedError)?;
        match claim_type_enum {
            ClaimType::Permissionless => {
                require!(self.locker == Pubkey::default(), ErrorCode::InvalidLocker);
                require!(
                    self.operator == Pubkey::default(),
                    ErrorCode::InvalidOperator
                );
            }
            ClaimType::Permissioned => {
                require!(self.locker == Pubkey::default(), ErrorCode::InvalidLocker);
            }
            ClaimType::PermissionlessWithStaking => {
                require!(self.locker != Pubkey::default(), ErrorCode::InvalidLocker);
                require!(
                    self.operator == Pubkey::default(),
                    ErrorCode::InvalidOperator
                );
            }
            ClaimType::PermissionedWithStaking => {
                require!(self.locker != Pubkey::default(), ErrorCode::InvalidLocker);
            }
        }
        Ok(())
    }
}
/// Accounts for [merkle_distributor::handle_new_distributor].
#[derive(Accounts)]
#[instruction(version: u64, total_node: u8)]
pub struct NewDistributor<'info> {
    /// [MerkleDistributor].
    #[account(
        init,
        seeds = [
            b"MerkleDistributor".as_ref(),
            base.key().to_bytes().as_ref(),
            mint.key().to_bytes().as_ref(),
            version.to_le_bytes().as_ref()
        ],
        bump,
        space = 8 + MerkleDistributor::INIT_SPACE,
        payer = payer
    )]
    pub distributor: AccountLoader<'info, MerkleDistributor>,

    /// The [DistributorRoot].
    #[account(mut)]
    pub distributor_root: AccountLoader<'info, DistributorRoot>,

    /// Base key of the distributor.
    pub base: Signer<'info>,

    /// Clawback receiver token account
    #[account(mut, token::mint = mint)]
    pub clawback_receiver: Account<'info, TokenAccount>,

    /// The mint to distribute.
    pub mint: Account<'info, Mint>,

    /// Token vault
    /// Should create previously
    #[account(
        init_if_needed,
        associated_token::mint = mint,
        associated_token::authority=distributor,
        payer = payer
    )]
    pub token_vault: Account<'info, TokenAccount>,

    /// The authority to set the clawback receiver and change itself.
    /// CHECK: This account is not use to read or write
    pub admin: UncheckedAccount<'info>,

    /// Payer wallet, responsible for creating the distributor and paying for the transaction.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The [System] program.
    pub system_program: Program<'info, System>,

    /// The [Token] program.
    pub token_program: Program<'info, Token>,

    // Associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

/// Creates a new [MerkleDistributor].
/// After creating this [MerkleDistributor],
/// the token_vault should be seeded with max_total_claim tokens.
/// CHECK:
///     1. The start timestamp is before the end timestamp
///     2. The clawback timestamp is after the end timestamp
///     3. The start, end, and clawback_start timestamps are all in the future
///     4. The clawback start is at least one day after end timestamp
#[allow(clippy::too_many_arguments)]
#[allow(clippy::result_large_err)]
pub fn handle_new_distributor(
    ctx: Context<NewDistributor>,
    params: &NewDistributorParams,
) -> Result<()> {
    params.validate()?;
    let mut distributor = ctx.accounts.distributor.load_init()?;
    distributor.bump = *ctx.bumps.get("distributor").unwrap();
    distributor.version = params.version;
    distributor.mint = ctx.accounts.mint.key();
    distributor.token_vault = ctx.accounts.token_vault.key();
    distributor.max_total_claim = params.get_max_total_claim()?;
    distributor.max_num_nodes = params.max_num_nodes;
    distributor.total_amount_claimed = 0;
    distributor.num_nodes_claimed = 0;
    distributor.start_ts = params.start_vesting_ts;
    distributor.end_ts = params.end_vesting_ts;
    distributor.clawback_start_ts = params.clawback_start_ts;
    distributor.clawback_receiver = ctx.accounts.clawback_receiver.key();
    distributor.admin = ctx.accounts.admin.key();
    distributor.clawed_back = 0;
    if params.closable {
        distributor.closable = 1;
    }
    distributor.base = ctx.accounts.base.key();
    distributor.airdrop_bonus = params.get_airdrop_bonus();
    distributor.claim_type = params.claim_type;
    distributor.activation_point = params.activation_point;
    distributor.activation_type = params.activation_type;
    distributor.operator = params.operator;
    distributor.locker = params.locker;
    distributor.distributor_root = ctx.accounts.distributor_root.key();

    // Note: might get truncated, do not rely on
    msg! {
        "New distributor created with version = {}, mint={}, vault={} max_total_claim={}, max_nodes: {}, start_ts: {}, end_ts: {}, clawback_start: {}, clawback_receiver: {} activation_point {} activation_type {} total_bonus {}, bonus_vesting_duration {}, claim_type {}",
            distributor.version,
            distributor.mint,
            ctx.accounts.token_vault.key(),
            distributor.max_total_claim,
            distributor.max_num_nodes,
            distributor.start_ts,
            distributor.end_ts,
            distributor.clawback_start_ts,
            distributor.clawback_receiver,
            distributor.activation_point,
            distributor.activation_type,
            distributor.airdrop_bonus.total_bonus,
            distributor.airdrop_bonus.vesting_duration,
            distributor.claim_type,
    };

    drop(distributor);

    // increase total distributor created
    let mut distributor_root = ctx.accounts.distributor_root.load_mut()?;
    distributor_root.update_new_distributor()?;

    Ok(())
}
