use crate::prelude::*;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use mpl_core::instructions::CreateCollectionV2CpiBuilder;
use mpl_core::types::{Plugin, PluginAuthority, PluginAuthorityPair, UpdateDelegate};

#[derive(Accounts)]
pub struct InitializeCollection<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = MasterState::SPACE,
        seeds = [b"master", collection.key().as_ref()],
        bump
    )]
    pub master_state: Account<'info, MasterState>,

    /// CHECK: Mint authority
    #[account(
        mut,
        seeds = [b"mint_authority", collection.key().as_ref()],
        bump
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: Initialized by MPL Core
    #[account(mut)]
    pub collection: Signer<'info>,

    /// CHECK Server authority
    #[account(
        mut,
        signer,
        constraint = update_authority.key() == SERVER_AUTHORITY @ CustomError::InvalidUpdateAuthority
    )]
    pub update_authority: AccountInfo<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    /// CHECK: MPL Core program
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CollectionArgs {
    pub name: String,
    pub uri: String,
}

pub fn handler(ctx: Context<InitializeCollection>, args: CollectionArgs) -> Result<()> {
    if ctx.accounts.mpl_core.key() != mpl_core::ID {
        return Err(ProgramError::IncorrectProgramId.into());
    }

    if ctx.accounts.system_program.key() != system_program::ID {
        return Err(ProgramError::IncorrectProgramId.into());
    }

    let delegate_plugin = PluginAuthorityPair {
        plugin: Plugin::UpdateDelegate(UpdateDelegate {
            additional_delegates: vec![ctx.accounts.mint_authority.key()],
        }),
        authority: Some(PluginAuthority::UpdateAuthority),
    };

    CreateCollectionV2CpiBuilder::new(&ctx.accounts.mpl_core.to_account_info())
        .collection(&ctx.accounts.collection.to_account_info())
        .update_authority(Some(ctx.accounts.update_authority.as_ref()))
        .payer(&ctx.accounts.payer.to_account_info())
        .system_program(&ctx.accounts.system_program.to_account_info())
        .name(args.name)
        .uri(args.uri)
        .plugins(vec![delegate_plugin])
        .invoke()?;

    let master_state = &mut ctx.accounts.master_state;
    master_state.total_minted = 0;
    master_state.collection = ctx.accounts.collection.key();

    Ok(())
}
