use anchor_lang::prelude::*;

use crate::{
    constants::ESCROW_SEED,
    error::EscrowError,
    state::Escrow,
};

#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        mut,
        seeds = [
            ESCROW_SEED,
            seller.key().as_ref(),
            escrow.offer_id.to_le_bytes().as_ref()
        ],
        bump = escrow.bump,
        has_one = seller @ EscrowError::Unauthorized
    )]
    pub escrow: Account<'info, Escrow>,

    pub system_program: Program<'info, System>,
}

pub fn handle_deposit_sol(ctx: Context<DepositSol>) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow;

    require!(
        escrow.status == 0,
        EscrowError::InvalidStatus
    );

    let cpi_accounts = anchor_lang::system_program::Transfer {
        from: ctx.accounts.seller.to_account_info(),
        to: escrow.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(
        ctx.accounts.system_program.key(),
        cpi_accounts,
    );

    anchor_lang::system_program::transfer(
        cpi_ctx,
        escrow.sol_amount,
    )?;

    escrow.status = 1;

    msg!("SOL deposited into escrow");

    Ok(())
}