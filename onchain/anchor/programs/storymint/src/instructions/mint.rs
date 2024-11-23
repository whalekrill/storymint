use crate::prelude::*;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use mpl_core::accounts::BaseCollectionV1;
use mpl_core::instructions::CreateV2CpiBuilder;

#[derive(Accounts)]
pub struct MintAsset<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = TokenVault::SPACE,
        seeds = [b"vault", asset.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, TokenVault>,

    #[account(mut)]
    pub asset: Signer<'info>,

    #[account(
        mut,
        seeds = [b"master", collection.key().as_ref()],
        bump,
        constraint = master_state.total_minted < MAX_SUPPLY @ CustomError::MaxSupplyReached,
        has_one = collection @ CustomError::InvalidCollection
    )]
    pub master_state: Account<'info, MasterState>,

    /// CHECK: Collection the asset belongs to
    #[account(mut)]
    pub collection: Account<'info, BaseCollectionV1>,

    /// CHECK: Mint authority
    #[account(
        seeds = [
            b"mint_authority",
            collection.key().as_ref()
        ],
        bump
    )]
    pub mint_authority: AccountInfo<'info>,

    /// CHECK: Asset owner
    pub owner: AccountInfo<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: MPL Core program
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,
}

pub fn handler(ctx: Context<MintAsset>) -> Result<()> {
    let rent = Rent::get()?;
    let vault_rent = rent.minimum_balance(TokenVault::SPACE);
    let total_required = VAULT_AMOUNT + vault_rent;

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ),
        total_required,
    )?;

    ctx.accounts.vault.asset = ctx.accounts.asset.key();

    let collection = ctx.accounts.collection.key();
    let authority_seeds = [
        b"mint_authority",
        collection.as_ref(),
        &[ctx.bumps.mint_authority],
    ];

    // Create the asset using MPL Core
    CreateV2CpiBuilder::new(&ctx.accounts.mpl_core)
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(ctx.accounts.collection.as_ref()))
        .authority(Some(&ctx.accounts.mint_authority.to_account_info()))
        .payer(ctx.accounts.payer.as_ref())
        .owner(Some(&ctx.accounts.payer.to_account_info()))
        .system_program(ctx.accounts.system_program.as_ref())
        .name(ctx.accounts.collection.name.clone())
        .uri(ctx.accounts.collection.uri.clone())
        .invoke_signed(&[&authority_seeds])?;

    ctx.accounts.master_state.total_minted = ctx
        .accounts
        .master_state
        .total_minted
        .checked_add(1)
        .ok_or(CustomError::Overflow)?;

    Ok(())
}
