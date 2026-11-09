use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub seller: Pubkey,
    pub buyer: Option<Pubkey>,
    pub usdc_mint: Pubkey,
    pub offer_id: u64,
    pub sol_amount: u64,
    pub usdc_amount: u64,
    pub bump: u8,
    pub status: u8,
}
