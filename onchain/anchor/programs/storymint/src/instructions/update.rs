use crate::prelude::*;
use anchor_lang::prelude::*;
use mpl_core::instructions::UpdateV1CpiBuilder;

#[derive(Accounts)]
pub struct UpdateMetadata<'info> {
    /// CHECK: Asset to be updated
    #[account(mut)]
    pub asset: AccountInfo<'info>,

    /// CHECK: Collection the asset belongs to
    #[account(mut)]
    pub collection: Option<AccountInfo<'info>>,

    #[account(
        mut, 
        signer,
        constraint = authority.key() == SERVER_AUTHORITY @ CustomError::UnauthorizedUpdate
    )]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: MPL Core program
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct UpdateMetadataArgs {
    pub name: Option<String>,
    pub uri: String,
}

pub fn handler(ctx: Context<UpdateMetadata>, args: UpdateMetadataArgs) -> Result<()> {
    UpdateV1CpiBuilder::new(&ctx.accounts.mpl_core)
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(ctx.accounts.collection.as_ref())
        .authority(Some(&ctx.accounts.authority.as_ref()))
        .payer(&ctx.accounts.payer.to_account_info())
        .system_program(&ctx.accounts.system_program.to_account_info())
        .new_name(args.name.ok_or(CustomError::InvalidMetadataUpdate)?)
        .new_uri(args.uri)
        .invoke()?;

    Ok(())
}
