use anchor_lang::{
    accounts::{account::Account, program::Program, signer::Signer},
    context::{Context, CpiContext},
    prelude::*,
    Accounts, Result, ToAccountInfo,
};
use anchor_spl::token::{self, Token, TokenAccount};

use crate::{
    error::ErrorCode,
    math::SafeMath,
    state::{merkle_distributor::MerkleDistributor, parent_account::ParentAccount},
};

/// Accounts required for distributing tokens from the parent vault to distributor vaults.
#[derive(Accounts)]
pub struct DistributeVault<'info> {
    /// The [ParentState].
    #[account(mut, has_one = admin)]
    pub parent_account: AccountLoader<'info, ParentAccount>,

    /// Parent vault containing the tokens to distribute to distributor vault.
    #[account(
        mut,
        associated_token::mint = parent_account.load()?.mint,
        associated_token::authority = parent_account.key(),
        address = parent_account.load()?.parent_vault,
    )]
    pub parent_vault: Account<'info, TokenAccount>,

    /// Admin
    pub admin: Signer<'info>,

    /// SPL [Token] program.
    pub token_program: Program<'info, Token>,
}

/// Handles the distribution of tokens from the parent vault to multiple distributor vaults.
///
/// # Accounts
/// - `parent_account`: The parent account containing distribution configurations.
/// - `parent_vault`: The token account holding the tokens to be distributed.
/// - `admin`: The admin authority initiating the distribution.
/// - `token_program`: The SPL token program.
///
/// # Remaining Accounts
/// The remaining accounts should be provided in pairs:
/// - `distributor_vault`: The token account of the distributor.
/// - `distributor`: The MerkleDistributor account.
pub fn handle_distribute_vault<'info>(
    ctx: Context<'_, '_, '_, 'info, DistributeVault<'info>>,
) -> Result<()> {
    let mut parent_account = ctx.accounts.parent_account.load_mut()?;
    let signer = parent_account.signer();
    let seeds = signer.seeds();

    // Remaining account layout
    // [distributor_vault, distributor, distributor_vault, distributor]
    let remaining_accounts: &[AccountInfo] = ctx.remaining_accounts;
    // Ensure valid layout.
    if remaining_accounts.len() % 2 != 0 {
        return Err(ErrorCode::InvalidRemainingAccounts.into());
    }

    for pair_accounts in remaining_accounts.chunks(2) {
        let distributor_vault = Account::<TokenAccount>::try_from(&pair_accounts[0])?;
        let distributor_account = AccountLoader::<MerkleDistributor>::try_from(&pair_accounts[1])?;
        let distributor_state = distributor_account.load()?;

        // Validate distributor vault is valid with distributor
        require_keys_eq!(
            distributor_vault.key(),
            distributor_state.token_vault,
            ErrorCode::InvalidAccount
        );

        // Validate distributor is valid with parent state
        require_keys_eq!(
            ctx.accounts.parent_account.key(),
            distributor_state.parent_account.key(),
            ErrorCode::InvalidAccount
        );

        // Check distributor has been funded token
        if distributor_vault.amount >= distributor_state.max_total_claim {
            msg!(
                "Airdrop already funded to version {}!",
                distributor_state.version
            );
            continue;
        }

        let amount_transfer = distributor_state
            .max_total_claim
            .safe_sub(distributor_vault.amount)?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.parent_vault.to_account_info(),
                    to: distributor_vault.to_account_info(),
                    authority: ctx.accounts.parent_account.to_account_info(),
                },
            )
            .with_signer(&[&seeds[..]]),
            amount_transfer,
        )?;

        msg!(
            "Transferred {} tokens to distributor version {}.",
            amount_transfer,
            distributor_state.version
        );

        // Accumulate distributed token amount
        parent_account.accumulate_distribution(amount_transfer)?;
    }

    drop(parent_account);

    Ok(())
}
