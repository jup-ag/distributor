use anchor_lang::{
    accounts::{account::Account, program::Program},
    context::Context,
    prelude::*,
    Accounts, Result, ToAccountInfo,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount},
};

use crate::state::distributor_root::DistributorRoot;

/// Accounts required for distributing tokens from the parent vault to distributor vaults.
#[derive(Accounts)]
pub struct FundDistributorRoot<'info> {
    /// The [DistributorRoot]
    #[account(mut, has_one = mint)]
    pub distributor_root: AccountLoader<'info, DistributorRoot>,

    /// Distributor root vault
    #[account(
        init_if_needed,
        associated_token::mint = mint,
        associated_token::authority = distributor_root,
        payer = payer
    )]
    pub distributor_root_vault: Account<'info, TokenAccount>,

    /// The mint to distribute.
    pub mint: Account<'info, Mint>,

    /// Payer.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Payer Token Account.
    #[account(mut)]
    pub payer_token: Account<'info, TokenAccount>,

    /// The [System] program.
    pub system_program: Program<'info, System>,

    /// The [Token] program.
    pub token_program: Program<'info, Token>,

    // Associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handle_fund_distributor_root(
    ctx: Context<FundDistributorRoot>,
    max_amount: u64,
) -> Result<()> {
    let fund_amount = {
        let mut distributor_root = ctx.accounts.distributor_root.load_mut()?;
        distributor_root.get_and_set_fund_amount(max_amount)?
    };

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.payer_token.to_account_info(),
                to: ctx.accounts.distributor_root_vault.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        fund_amount,
    )?;

    Ok(())
}
