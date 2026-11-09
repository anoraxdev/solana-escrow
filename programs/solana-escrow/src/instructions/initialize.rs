use anchor_lang::prelude::*;

use crate::{constants::ESCROW_SEED, state::Escrow};

#[derive(Accounts)]
#[instruction(offer_id: u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        init,
        payer = seller,
        space = 8 + Escrow::INIT_SPACE,
        seeds = [
            ESCROW_SEED,
            seller.key().as_ref(),
            offer_id.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub escrow: Account<'info, Escrow>,

    pub system_program: Program<'info, System>,
}

pub fn handle_initialize(
    ctx: Context<Initialize>,
    offer_id: u64,
    sol_amount: u64,
    usdc_amount: u64,
    usdc_mint: Pubkey,
) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow;

    escrow.seller = ctx.accounts.seller.key();
    escrow.buyer = None;
    escrow.usdc_mint = usdc_mint;
    escrow.offer_id = offer_id;
    escrow.sol_amount = sol_amount;
    escrow.usdc_amount = usdc_amount;
    escrow.bump = ctx.bumps.escrow;
    escrow.status = 0;

    msg!("Escrow offer initialized");

    Ok(())
}