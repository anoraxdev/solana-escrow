use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Unauthorized access")]
    Unauthorized,

    #[msg("Invalid escrow status")]
    InvalidStatus,
}