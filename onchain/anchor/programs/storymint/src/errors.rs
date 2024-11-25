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
    #[msg("Insufficient funds for minting")]
    InsufficientFunds,
    #[msg("Invalid MPL Core program address")]
    InvalidMplCoreProgram,
    #[msg("Invalid owner signature")]
    InvalidOwner,
    #[msg("Rent calculation failed")]
    RentCalculationError,
    #[msg("Invalid token vault initialization")]
    InvalidVaultInit,
    #[msg("System transfer failed")]
    TransferFailed,
    #[msg("Invalid metadata parameters")]
    InvalidMetadataParams,
    #[msg("Asset creation failed")]
    AssetCreationFailed,
    #[msg("Invalid PDA derivation")]
    InvalidPdaDerivation,
    #[msg("State update failed")]
    StateUpdateFailed,
}
