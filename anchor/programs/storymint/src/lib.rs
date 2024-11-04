use anchor_lang::prelude::*;

pub mod prelude {
    pub use crate::constants::*;
    pub use crate::errors::*;
    pub use crate::state::*;
}

mod constants;
mod errors;
mod instructions;
mod state;

use crate::instructions::*;

declare_id!("5qGtv9tb8yWuLG6jm2EPTBa3SsLP2hkhToB7tZT6mp5B");

#[program]
pub mod storymint {
    use super::*;

    pub fn initialize_collection(
        ctx: Context<InitializeCollection>,
        args: CollectionArgs,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, args)
    }

    pub fn mint_asset(ctx: Context<MintAsset>) -> Result<()> {
        instructions::mint::handler(ctx)
    }

    pub fn update_metadata(ctx: Context<UpdateMetadata>, args: UpdateMetadataArgs) -> Result<()> {
        instructions::update::handler(ctx, args)
    }

    pub fn burn_and_withdraw(ctx: Context<BurnAndWithdraw>) -> Result<()> {
        instructions::burn::handler(ctx)
    }
}
