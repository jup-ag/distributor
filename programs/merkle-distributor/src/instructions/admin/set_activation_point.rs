use crate::state::merkle_distributor::MerkleDistributor;
use anchor_lang::{context::Context, prelude::*, Accounts, Key, Result};
/// Accounts for [merkle_distributor::set_activation_point].
#[derive(Accounts)]
pub struct SetActivationPoint<'info> {
    /// [MerkleDistributor].
    #[account(
        mut,
        has_one = admin,
    )]
    pub distributor: AccountLoader<'info, MerkleDistributor>,

    /// Payer to create the distributor.
    #[account(mut)]
    pub admin: Signer<'info>,
}

/// set activation point
#[allow(clippy::result_large_err)]
pub fn handle_set_activation_point(
    ctx: Context<SetActivationPoint>,
    activation_point: u64,
) -> Result<()> {
    let mut distributor = ctx.accounts.distributor.load_mut()?;
    distributor.activation_point = activation_point;
    Ok(())
}
