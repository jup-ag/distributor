use crate::state::parent_account::ParentAccount;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

#[derive(Accounts)]
pub struct NewParentAccount<'info> {
    /// [ParentAccount]
    #[account(
        init,
        seeds = [
            b"ParentAccount".as_ref(),
            mint.key().to_bytes().as_ref(),
        ],
        bump,
        space = 8 + ParentAccount::INIT_SPACE,
        payer = admin
    )]
    pub parent_account: AccountLoader<'info, ParentAccount>,

    /// Parent vault
    /// Should create previously
    #[account(
        associated_token::mint = mint,
        associated_token::authority = parent_account,
    )]
    pub parent_vault: Account<'info, TokenAccount>,

    /// The mint to distribute.
    pub mint: Account<'info, Mint>,

    /// Admin wallet, responsible for creating the distributor and paying for the transaction.
    /// Also has the authority to set the clawback receiver and change itself.
    #[account(mut)]
    pub admin: Signer<'info>,

    /// The [System] program.
    pub system_program: Program<'info, System>,
}

pub fn handle_new_parent_account(ctx: Context<NewParentAccount>) -> Result<()> {
    let mut parent_account = ctx.accounts.parent_account.load_init()?;

    parent_account.bump = *ctx.bumps.get("parent_account").unwrap();
    parent_account.admin = ctx.accounts.admin.key();
    parent_account.mint = ctx.accounts.mint.key();
    parent_account.parent_vault = ctx.accounts.parent_vault.key();

    Ok(())
}
