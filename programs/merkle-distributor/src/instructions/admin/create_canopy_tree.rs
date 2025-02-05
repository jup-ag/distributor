use crate::{
    error::ErrorCode,
    state::{canopy_tree::CanopyTree, merkle_distributor::MerkleDistributor},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(depth: u8)]
pub struct CreateCanopyTree<'info> {
    /// [CanopyTree]
    #[account(
        init,
        seeds = [
            b"CanopyTree".as_ref(),
            distributor.key().to_bytes().as_ref(),
        ],
        bump,
        space = CanopyTree::space(depth as usize),
        payer = admin
    )]
    pub canopy_tree: Account<'info, CanopyTree>,

    /// The [MerkleDistributor].
    pub distributor: AccountLoader<'info, MerkleDistributor>,

    #[account(mut)]
    pub admin: Signer<'info>,

    /// The [System] program.
    pub system_program: Program<'info, System>,
}

pub fn handle_create_canopy_tree(
    ctx: Context<CreateCanopyTree>,
    depth: u8,
    root: [u8; 32],
    canopy_nodes: Vec<[u8; 32]>,
) -> Result<()> {
    let canopy_tree = &mut ctx.accounts.canopy_tree;

    let verify_canopy_root = canopy_tree.verify_canopy_root(root, canopy_nodes.clone());
    require!(verify_canopy_root, ErrorCode::CanopyRootMissMatch);

    canopy_tree.root = root;
    canopy_tree.depth = depth;
    canopy_tree.nodes = canopy_nodes;
    canopy_tree.distributor = ctx.accounts.distributor.key();

    Ok(())
}
