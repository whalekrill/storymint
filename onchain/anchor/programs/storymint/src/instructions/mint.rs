use crate::prelude::*;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use mpl_core::accounts::BaseCollectionV1;
use mpl_core::instructions::CreateV2CpiBuilder;

#[derive(Accounts)]
#[instruction(mint_authority_bump: u8)]
pub struct MintAsset<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = TokenVault::SPACE,
        seeds = [b"vault", asset.key().as_ref()],
        bump,
        rent_exempt = enforce  
    )]
    pub vault: Account<'info, TokenVault>,

    #[account(mut)]
    pub asset: Signer<'info>,

    #[account(
        mut,
        seeds = [b"master", collection.key().as_ref()],
        bump,
        constraint = master_state.total_minted < MAX_SUPPLY @ CustomError::MaxSupplyReached,
        constraint = master_state.collection == collection.key() @ CustomError::InvalidCollection,
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
        seeds::program = mpl_core::ID,
        bump,
        constraint = mint_authority.lamports() > 0 @ CustomError::InvalidAuthority
    )]
    pub mint_authority: AccountInfo<'info>,

    /// CHECK: Asset owner
    #[account(constraint = owner.is_signer @ CustomError::InvalidOwner)]
    pub owner: AccountInfo<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: MPL Core program with explicit check
    #[account(
        address = mpl_core::ID @ CustomError::InvalidMplCoreProgram
    )]
    pub mpl_core: AccountInfo<'info>,
}

pub fn handler(ctx: Context<MintAsset>) -> Result<()> {
    // Calculate and verify rent before transfer
    let rent = Rent::get()?;
    let vault_rent = rent.minimum_balance(TokenVault::SPACE);
    let total_required = VAULT_AMOUNT
        .checked_add(vault_rent)
        .ok_or(CustomError::Overflow)?;

    // Verify payer has sufficient balance
    require!(
        ctx.accounts.payer.lamports() >= total_required,
        CustomError::InsufficientFunds
    );

    // Transfer with explicit error handling
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

    // Initialize vault immediately after transfer
    ctx.accounts.vault.asset = ctx.accounts.asset.key();

    let collection = ctx.accounts.collection.key();
    let authority_seeds = [b"mint_authority", collection.as_ref(), &[ctx.bumps.vault]];

    // Create asset with explicit checks
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

    // Update state with checked math
    ctx.accounts.master_state.total_minted = ctx
        .accounts
        .master_state
        .total_minted
        .checked_add(1)
        .ok_or(CustomError::Overflow)?;

    Ok(())
}
