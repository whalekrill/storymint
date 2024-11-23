use anchor_lang::prelude::*;

#[account]
pub struct MasterState {
    pub collection: Pubkey,
    pub total_minted: u64,
}

impl MasterState {
    pub const SPACE: usize = 8 + 32 + 8; // discriminator + collection + total_minted
}

#[account]
#[derive(Default)]
pub struct TokenVault {
    pub mint: Pubkey,
}

impl TokenVault {
    pub const SPACE: usize = 8 + 32; // discriminator + mint
}
