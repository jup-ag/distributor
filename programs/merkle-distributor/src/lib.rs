//! A program for distributing tokens efficiently via uploading a [Merkle root](https://en.wikipedia.org/wiki/Merkle_tree).
//!
//! This program is largely based off of [Uniswap's Merkle Distributor](https://github.com/Uniswap/merkle-distributor).
//!
//! # Rationale
//!
//! Although Solana has low fees for executing transactions, it requires staking tokens to pay for storage costs, also known as "rent". These rent costs can add up when sending tokens to thousands or tens of thousands of wallets, making it economically unreasonable to distribute tokens to everyone.
//!
//! The Merkle distributor, pioneered by [Uniswap](https://github.com/Uniswap/merkle-distributor), solves this issue by deriving a 256-bit "root hash" from a tree of balances. This puts the gas cost on the claimer. Solana has the additional advantage of being able to reclaim rent from closed token accounts, so the net cost to the user should be around `0.000010 SOL` (at the time of writing).
//!
//! The Merkle distributor is also significantly easier to manage from an operations perspective, since one does not need to send a transaction to each individual address that may be redeeming tokens.

#![allow(clippy::too_many_arguments)]
use anchor_lang::prelude::*;
use instructions::*;

pub mod error;
pub mod instructions;
pub mod math;
pub mod state;
use solana_security_txt::security_txt;
declare_id!("DiS3nNjFVMieMgmiQFm6wgJL7nevk4NrhXKLbtEH1Z2R");

security_txt! {
    // Required fields
    name: "Merkle Distributor",
    project_url: "https://jup.ag/",
    contacts: "email:feedback.raccoons.dev",
    policy: "https://github.com/jup-ag/distributor",
    // Optional Fields
    preferred_languages: "en",
    source_code: "https://github.com/jup-ag/distributor"
}

#[program]
pub mod merkle_distributor {

    use super::*;

    /// ADMIN FUNCTIONS ////
    #[allow(clippy::result_large_err)]
    pub fn new_distributor_root(
        ctx: Context<NewDistributorRoot>,
        max_claim_amount: u64,
        max_distributor: u64,
    ) -> Result<()> {
        handle_new_distributor_root(ctx, max_claim_amount, max_distributor)
    }

    #[allow(clippy::result_large_err)]
    pub fn fund_distributor_root(ctx: Context<FundDistributorRoot>, max_amount: u64) -> Result<()> {
        handle_fund_distributor_root(ctx, max_amount)
    }

    #[allow(clippy::result_large_err)]
    pub fn new_distributor(
        ctx: Context<NewDistributor>,
        params: NewDistributorParams,
    ) -> Result<()> {
        handle_new_distributor(ctx, &params)
    }

    pub fn create_canopy_tree(
        ctx: Context<CreateCanopyTree>,
        depth: u8,
        root: [u8; 32],
        canopy_nodes: Vec<[u8; 32]>,
    ) -> Result<()> {
        handle_create_canopy_tree(ctx, depth, root, canopy_nodes)
    }

    #[allow(clippy::result_large_err)]
    pub fn fund_merkle_distributor_from_root<'info>(
        ctx: Context<'_, '_, '_, 'info, FundMerkleDisitributorFromRoot<'info>>,
    ) -> Result<()> {
        handle_fund_merkle_distributor_from_root(ctx)
    }

    /// only available in test phase
    #[allow(clippy::result_large_err)]
    pub fn close_distributor(ctx: Context<CloseDistributor>) -> Result<()> {
        handle_close_distributor(ctx)
    }
    /// only available in test phase
    #[allow(clippy::result_large_err)]
    pub fn close_claim_status(ctx: Context<CloseClaimStatus>) -> Result<()> {
        handle_close_status(ctx)
    }

    #[allow(clippy::result_large_err)]
    pub fn set_activation_point(
        ctx: Context<SetActivationPoint>,
        activation_point: u64,
    ) -> Result<()> {
        handle_set_activation_point(ctx, activation_point)
    }

    #[allow(clippy::result_large_err)]
    pub fn clawback(ctx: Context<Clawback>) -> Result<()> {
        handle_clawback(ctx)
    }

    #[allow(clippy::result_large_err)]
    pub fn set_clawback_receiver(ctx: Context<SetClawbackReceiver>) -> Result<()> {
        handle_set_clawback_receiver(ctx)
    }

    #[allow(clippy::result_large_err)]
    pub fn set_admin(ctx: Context<SetAdmin>) -> Result<()> {
        handle_set_admin(ctx)
    }

    #[allow(clippy::result_large_err)]
    pub fn set_operator(ctx: Context<SetOperator>, new_operator: Pubkey) -> Result<()> {
        handle_set_operator(ctx, new_operator)
    }

    //// END ADMIN FUNCTIONS ////
    /// USER FUNCTIONS /////
    #[allow(clippy::result_large_err)]
    pub fn new_claim(
        ctx: Context<NewClaim>,
        amount_unlocked: u64,
        amount_locked: u64,
        leaf_index: u32,
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        handle_new_claim(ctx, amount_unlocked, amount_locked, leaf_index, proof)
    }

    #[allow(clippy::result_large_err)]
    pub fn claim_locked(ctx: Context<ClaimLocked>) -> Result<()> {
        handle_claim_locked(ctx)
    }

    #[allow(clippy::result_large_err)]
    pub fn new_claim_and_stake(
        ctx: Context<NewClaimAndStake>,
        amount_unlocked: u64,
        amount_locked: u64,
        leaf_index: u32,
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        handle_new_claim_and_stake(ctx, amount_unlocked, amount_locked, leaf_index, proof)
    }

    #[allow(clippy::result_large_err)]
    pub fn claim_locked_and_stake(ctx: Context<ClaimLockedAndStake>) -> Result<()> {
        handle_claim_locked_and_stake(ctx)
    }
    // END USER FUNCTIONS //
}
