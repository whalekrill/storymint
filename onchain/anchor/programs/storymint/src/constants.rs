use anchor_lang::prelude::*;

pub const VAULT_AMOUNT: u64 = 1_000_000_000; // 1 SOL
pub const MAX_SUPPLY: u64 = 10_000;

#[cfg(not(feature = "mainnet"))]
pub const SERVER_AUTHORITY: Pubkey = pubkey!("EiLANmnffXVXczyimnGEKSZpzwQ4TyuQXVAviqBji8TF");

#[cfg(feature = "mainnet")]
pub const SERVER_AUTHORITY: Pubkey = pubkey!("ToDo44444444444444444444444444444444444444"); // TODO: Update with real mainnet address
