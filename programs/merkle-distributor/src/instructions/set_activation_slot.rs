use crate::error::ErrorCode;
use crate::state::merkle_distributor::ActivationType;
use crate::state::merkle_distributor::MerkleDistributor;
use anchor_lang::{context::Context, prelude::*, Accounts, Key, Result};
/// Accounts for [merkle_distributor::set_enable_slot].
#[derive(Accounts)]
pub struct SetActivationSlot<'info> {
    /// [MerkleDistributor].
    #[account(
        mut,
        has_one = admin,
    )]
    pub distributor: Account<'info, MerkleDistributor>,

    /// Payer to create the distributor.
    #[account(mut)]
    pub admin: Signer<'info>,
}

/// set enable slot
#[allow(clippy::result_large_err)]
pub fn handle_set_activation_slot(
    ctx: Context<SetActivationSlot>,
    activation_slot: u64,
) -> Result<()> {
    let distributor = &mut ctx.accounts.distributor;
    let activation_type = ActivationType::try_from(distributor.activation_type).unwrap();
    require!(
        activation_type == ActivationType::Slot,
        ErrorCode::InvalidActivationType
    );

    distributor.activation_slot = activation_slot;
    Ok(())
}
