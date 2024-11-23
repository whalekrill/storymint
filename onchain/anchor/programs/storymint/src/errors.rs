use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("Invalid vault balance")]
    InvalidVaultBalance,
    #[msg("Unauthorized metadata update")]
    UnauthorizedUpdate,
    #[msg("Maximum supply reached")]
    MaxSupplyReached,
    #[msg("Invalid collection data")]
    InvalidCollection,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Arithmetic underflow")]
    Underflow,
    #[msg("Invalid update authority")]
    InvalidUpdateAuthority,
    #[msg("Invalid metadata update")]
    InvalidMetadataUpdate,
}
