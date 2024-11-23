use crate::constants::*;
use anchor_lang::prelude::*;

pub const MINT_SPACE: usize = 82;

pub struct ProgramRent {
    pub vault: u64,
    pub mint: u64,
    pub metadata: u64,
}

pub fn calculate_rent(rent: &Rent, include_vault_amount: bool) -> ProgramRent {
    let vault = if include_vault_amount {
        VAULT_AMOUNT + rent.minimum_balance(TokenVault::SPACE)
    } else {
        rent.minimum_balance(TokenVault::SPACE)
    };

    ProgramRent {
        vault,
        mint: rent.minimum_balance(MINT_SPACE),
        metadata: rent.minimum_balance(METADATA_SIZE),
    }
}
