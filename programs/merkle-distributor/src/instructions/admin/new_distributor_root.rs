use crate::state::distributor_root::DistributorRoot;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct NewDistributorRoot<'info> {
    /// [DistributorRoot]
    #[account(
        init,
        seeds = [
            b"DistributorRoot".as_ref(),
            base.key().to_bytes().as_ref(),
            mint.key().to_bytes().as_ref(),
        ],
        bump,
        space = 8 + DistributorRoot::INIT_SPACE,
        payer = admin
    )]
    pub distributor_root: AccountLoader<'info, DistributorRoot>,

    /// Distributor root vault
    #[account(
        init_if_needed,
        associated_token::mint = mint,
        associated_token::authority = distributor_root,
        payer = admin
    )]
    pub distributor_root_vault: Account<'info, TokenAccount>,

    /// The mint to distribute.
    pub mint: Account<'info, Mint>,

    /// Base key of the distributor.
    pub base: Signer<'info>,

    /// Admin wallet, responsible for creating the distributor and paying for the transaction.
    /// Also has the authority to set the clawback receiver and change itself.
    #[account(mut)]
    pub admin: Signer<'info>,

    /// The [System] program.
    pub system_program: Program<'info, System>,

    /// The [Token] program.
    pub token_program: Program<'info, Token>,

    // Associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handle_new_distributor_root(
    ctx: Context<NewDistributorRoot>,
    max_claim_amount: u64,
    max_distributor: u64,
) -> Result<()> {
    let mut distributor_root = ctx.accounts.distributor_root.load_init()?;

    distributor_root.bump = *ctx.bumps.get("distributor_root").unwrap();
    distributor_root.mint = ctx.accounts.mint.key();
    distributor_root.base = ctx.accounts.base.key();
    distributor_root.max_claim_amount = max_claim_amount;
    distributor_root.max_distributor = max_distributor;
    distributor_root.distributor_root_vault = ctx.accounts.distributor_root_vault.key();

    Ok(())
}
