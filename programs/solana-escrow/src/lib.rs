pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("5esijGUsZavZQEJ9HCMxpotAM4fDkF5hj2vQTty4eBLx");

#[program]
pub mod solana_escrow {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        offer_id: u64,
        sol_amount: u64,
        usdc_amount: u64,
        usdc_mint: Pubkey,
    ) -> Result<()> {
        crate::instructions::initialize::handle_initialize(
            ctx,
            offer_id,
            sol_amount,
            usdc_amount,
            usdc_mint,
        )
    }

    pub fn deposit_sol(ctx: Context<DepositSol>) -> Result<()> {
        crate::instructions::deposit_sol::handle_deposit_sol(ctx)
    }
}
