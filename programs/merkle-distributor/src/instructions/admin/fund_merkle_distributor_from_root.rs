use anchor_lang::{
    accounts::{account::Account, program::Program},
    context::{Context, CpiContext},
    prelude::*,
    Accounts, Result, ToAccountInfo,
};
use anchor_spl::token::{self, Token, TokenAccount};

use crate::state::{distributor_root::DistributorRoot, merkle_distributor::MerkleDistributor};

/// Accounts required for distributing tokens from the parent vault to distributor vaults.
#[derive(Accounts)]
pub struct FundMerkleDisitributorFromRoot<'info> {
    /// The [DistributorRoot].
    pub distributor_root: AccountLoader<'info, DistributorRoot>,

    /// Distributor root vault containing the tokens to distribute to distributor vault.
    #[account(
        mut,
        associated_token::mint = distributor_root.load()?.mint,
        associated_token::authority = distributor_root.key(),
        address = distributor_root.load()?.distributor_root_vault,
    )]
    pub distributor_root_vault: Account<'info, TokenAccount>,

    /// The [MerkleDistributor].
    #[account(mut, constraint = distributor.load()?.distributor_root == distributor_root.key())]
    pub distributor: AccountLoader<'info, MerkleDistributor>,

    /// Distributor vault
    #[account(
        mut,
        associated_token::mint = distributor.load()?.mint,
        associated_token::authority = distributor.key(),
    )]
    pub distributor_vault: Account<'info, TokenAccount>,

    /// SPL [Token] program.
    pub token_program: Program<'info, Token>,
}

/// Handles the distribution of tokens from the parent vault to multiple distributor vaults.
pub fn handle_fund_merkle_distributor_from_root<'info>(
    ctx: Context<'_, '_, '_, 'info, FundMerkleDisitributorFromRoot<'info>>,
) -> Result<()> {
    let distributor_root = ctx.accounts.distributor_root.load()?;
    let signer = distributor_root.signer();
    let seeds = signer.seeds();

    let mut distributor_state = ctx.accounts.distributor.load_mut()?;

    // Check distributor has been funded token
    if distributor_state.funded_amount == 0 {
        let fund_amount = distributor_state.max_total_claim;
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.distributor_root_vault.to_account_info(),
                    to: ctx.accounts.distributor_vault.to_account_info(),
                    authority: ctx.accounts.distributor_root.to_account_info(),
                },
            )
            .with_signer(&[&seeds[..]]),
            fund_amount,
        )?;

        distributor_state.accumulate_funded_amount(fund_amount)?;

        msg!(
            "Funded {} tokens to distributor version {}.",
            fund_amount,
            distributor_state.version
        );
    }

    Ok(())
}
