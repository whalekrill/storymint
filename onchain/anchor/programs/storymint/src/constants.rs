use anchor_lang::prelude::*;

pub const NAME: &str = "Locked SOL NFT";
pub const SYMBOL: &str = "LSOL";
pub const URI: &str = "https://api.locked-sol.com/metadata/initial.json";
pub const SELLER_FEE_BASIS_POINTS: u16 = 0;

pub const VAULT_AMOUNT: u64 = 1_000_000_000; // 1 SOL
pub const MAX_SUPPLY: u64 = 10_000;
pub const METADATA_SIZE: usize = 679;

#[cfg(not(feature = "mainnet"))]
pub const SERVER_AUTHORITY: Pubkey = pubkey!("EiLANmnffXVXczyimnGEKSZpzwQ4TyuQXVAviqBji8TF");

#[cfg(feature = "mainnet")]
pub const SERVER_AUTHORITY: Pubkey = pubkey!("ToDo44444444444444444444444444444444444444");
