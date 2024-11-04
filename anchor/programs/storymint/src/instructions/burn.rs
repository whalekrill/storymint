use crate::prelude::*;
use anchor_lang::prelude::*;
use mpl_core::instructions::BurnV1CpiBuilder;

#[derive(Accounts)]
pub struct BurnAndWithdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    /// CHECK: Asset to be burned
    #[account(mut)]
    pub asset: AccountInfo<'info>,

    /// CHECK: Collection the asset belongs to
    #[account(mut)]
    pub collection: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"master", collection.key().as_ref()],
        bump,
        has_one = collection @ CustomError::InvalidCollection
    )]
    pub master_state: Account<'info, MasterState>,

    #[account(
        mut,
        seeds = [b"vault", asset.key().as_ref()],
        bump,
        close = owner
    )]
    pub vault: Account<'info, TokenVault>,

    pub system_program: Program<'info, System>,

    /// CHECK: MPL Core program
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,
}

pub fn handler(ctx: Context<BurnAndWithdraw>) -> Result<()> {
    if ctx.accounts.mpl_core.key() != mpl_core::ID {
        return Err(ProgramError::IncorrectProgramId.into());
    }

    // Burn the asset
    BurnV1CpiBuilder::new(&ctx.accounts.mpl_core)
        .payer(&ctx.accounts.owner)
        .asset(&ctx.accounts.asset)
        .collection(Some(&ctx.accounts.collection))
        .invoke()?;

    ctx.accounts.master_state.reload()?;
    // Update collection stats
    ctx.accounts.master_state.total_minted = ctx
        .accounts
        .master_state
        .total_minted
        .checked_sub(1)
        .ok_or(CustomError::Underflow)?;

    Ok(())
}
