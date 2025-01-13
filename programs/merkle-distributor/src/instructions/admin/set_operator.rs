use crate::{
    error::ErrorCode,
    state::merkle_distributor::{ClaimType, MerkleDistributor},
};
use anchor_lang::{
    accounts::signer::Signer, context::Context, prelude::*, Accounts, Result, ToAccountInfo,
};

/// [merkle_distributor::set_clawback_receiver] accounts.
#[derive(Accounts)]
pub struct SetOperator<'info> {
    /// The [MerkleDistributor].
    #[account(mut, has_one=admin)]
    pub distributor: AccountLoader<'info, MerkleDistributor>,

    /// Admin signer
    pub admin: Signer<'info>,
}

/// Sets new clawback receiver token account
/// CHECK:
///     1. The new clawback receiver is not the same as the old one
#[allow(clippy::result_large_err)]
pub fn handle_set_operator(ctx: Context<SetOperator>, new_operator: Pubkey) -> Result<()> {
    let mut distributor = ctx.accounts.distributor.load_mut()?;

    let claim_type =
        ClaimType::try_from(distributor.claim_type).map_err(|_| ErrorCode::TypeCastedError)?;
    require!(
        claim_type == ClaimType::Permissioned || claim_type == ClaimType::PermissionedWithStaking,
        ErrorCode::InvalidClaimType
    );
    require!(
        distributor.operator != new_operator,
        ErrorCode::SameOperator
    );

    distributor.operator = new_operator;
    Ok(())
}
