use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Token},
};

use mpl_token_metadata::instructions::{
    CreateMasterEditionV3, CreateMasterEditionV3InstructionArgs, CreateMetadataAccountV3,
    CreateMetadataAccountV3InstructionArgs,
};
use mpl_token_metadata::types::{Collection, DataV2};
use mpl_token_metadata::ID as METADATA_PROGRAM_ID;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[error_code]
pub enum CustomError {
    InvalidTokenAmount,
    InvalidVaultBalance,
    InvalidTokenAccount,
    TokenAccountNotClosed,
    MaxSupplyReached,
    Overflow,
}

const VAULT_AMOUNT: u64 = 1_000_000_000; // 1 SOL in lamports
const MAX_SUPPLY: u64 = 10_000; // Max NFTs in master edition

#[program]
pub mod locked_sol_pnft {
    use super::*;

    pub fn initialize_master_edition(
        ctx: Context<InitializeMasterEdition>,
        uri: String,
    ) -> Result<()> {
        // Initialize collection state
        ctx.accounts.edition_state.authority = ctx.accounts.authority.key();
        ctx.accounts.edition_state.master_mint = ctx.accounts.master_mint.key();
        ctx.accounts.edition_state.total_minted = 0;

        // Create seed bindings
        let master_mint_key = ctx.accounts.master_mint.key();
        let auth_seeds = &[
            b"master_authority".as_ref(),
            master_mint_key.as_ref(),
            &[ctx.bumps.master_authority],
        ];
        let auth_signer = &[&auth_seeds[..]];

        // Initialize master edition mint
        token::initialize_mint(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::InitializeMint {
                    mint: ctx.accounts.master_mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            0,
            &ctx.accounts.master_authority.key(),
            Some(&ctx.accounts.master_authority.key()),
        )?;

        // Create metadata
        let create_metadata_ix = CreateMetadataAccountV3 {
            metadata: ctx.accounts.master_metadata.key(),
            mint: ctx.accounts.master_mint.key(),
            mint_authority: ctx.accounts.master_authority.key(),
            payer: ctx.accounts.authority.key(),
            update_authority: (ctx.accounts.master_authority.key(), true),
            system_program: ctx.accounts.system_program.key(),
            rent: Some(ctx.accounts.rent.key()),
        }
        .instruction(CreateMetadataAccountV3InstructionArgs {
            data: DataV2 {
                name: "Locked SOL Collection".to_string(),
                symbol: "LSOL".to_string(),
                uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            is_mutable: false,
            collection_details: None,
        });

        anchor_lang::solana_program::program::invoke_signed(
            &create_metadata_ix,
            &[
                ctx.accounts.master_metadata.to_account_info(),
                ctx.accounts.master_mint.to_account_info(),
                ctx.accounts.master_authority.to_account_info(),
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
            auth_signer,
        )?;

        // Create master edition with max supply
        let create_master_edition_ix = CreateMasterEditionV3 {
            edition: ctx.accounts.master_edition.key(),
            mint: ctx.accounts.master_mint.key(),
            update_authority: ctx.accounts.master_authority.key(),
            mint_authority: ctx.accounts.master_authority.key(),
            metadata: ctx.accounts.master_metadata.key(),
            payer: ctx.accounts.authority.key(),
            token_program: ctx.accounts.token_program.key(),
            system_program: ctx.accounts.system_program.key(),
            rent: Some(ctx.accounts.rent.key()),
        }
        .instruction(CreateMasterEditionV3InstructionArgs {
            max_supply: Some(MAX_SUPPLY),
        });

        anchor_lang::solana_program::program::invoke_signed(
            &create_master_edition_ix,
            &[
                ctx.accounts.master_edition.to_account_info(),
                ctx.accounts.master_mint.to_account_info(),
                ctx.accounts.master_authority.to_account_info(),
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.master_metadata.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
            auth_signer,
        )?;

        Ok(())
    }

    pub fn mint_pnft(ctx: Context<MintPNFT>, metadata_uri: String) -> Result<()> {
        // Check we haven't exceeded max supply
        require!(
            ctx.accounts.edition_state.total_minted < MAX_SUPPLY,
            CustomError::MaxSupplyReached
        );

        // Transfer SOL to vault
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                },
            ),
            VAULT_AMOUNT,
        )?;

        // Initialize vault
        ctx.accounts.vault.owner = ctx.accounts.payer.key();
        ctx.accounts.vault.mint = ctx.accounts.mint.key();

        // Initialize mint with PDA authority
        let mint_bump = ctx.bumps.mint_authority;
        let mint_key = ctx.accounts.mint.key();
        let mint_seeds = &[b"mint_authority".as_ref(), mint_key.as_ref(), &[mint_bump]];
        let mint_authority_signer = &[&mint_seeds[..]];

        token::initialize_mint(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::InitializeMint {
                    mint: ctx.accounts.mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            0,
            &ctx.accounts.mint_authority.key(),
            Some(&ctx.accounts.mint_authority.key()),
        )?;

        // Create associated token account
        anchor_spl::associated_token::create(CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: ctx.accounts.payer.to_account_info(),
                associated_token: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
        ))?;

        // Mint NFT
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                mint_authority_signer,
            ),
            1,
        )?;

        // Create metadata with collection
        let create_metadata_ix = CreateMetadataAccountV3 {
            metadata: ctx.accounts.metadata.key(),
            mint: ctx.accounts.mint.key(),
            mint_authority: ctx.accounts.mint_authority.key(),
            payer: ctx.accounts.payer.key(),
            update_authority: (ctx.accounts.mint_authority.key(), true),
            system_program: ctx.accounts.system_program.key(),
            rent: Some(ctx.accounts.rent.key()),
        }
        .instruction(CreateMetadataAccountV3InstructionArgs {
            data: DataV2 {
                name: "Locked SOL NFT".to_string(),
                symbol: "LSOL".to_string(),
                uri: metadata_uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: Some(Collection {
                    verified: false, // Will be verified in a follow-up tx
                    key: ctx.accounts.edition_state.master_mint,
                }),
                uses: None,
            },
            is_mutable: false,
            collection_details: None,
        });

        anchor_lang::solana_program::program::invoke_signed(
            &create_metadata_ix,
            &[
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
            mint_authority_signer,
        )?;

        // Create edition marker
        let create_edition_ix = CreateMasterEditionV3 {
            edition: ctx.accounts.master_edition.key(),
            mint: ctx.accounts.mint.key(),
            update_authority: ctx.accounts.mint_authority.key(),
            mint_authority: ctx.accounts.mint_authority.key(),
            metadata: ctx.accounts.metadata.key(),
            payer: ctx.accounts.payer.key(),
            token_program: ctx.accounts.token_program.key(),
            system_program: ctx.accounts.system_program.key(),
            rent: Some(ctx.accounts.rent.key()),
        }
        .instruction(CreateMasterEditionV3InstructionArgs {
            max_supply: Some(0), // No editions for this NFT
        });

        anchor_lang::solana_program::program::invoke_signed(
            &create_edition_ix,
            &[
                ctx.accounts.master_edition.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
            mint_authority_signer,
        )?;

        // Revoke mint authority
        token::set_authority(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::SetAuthority {
                    current_authority: ctx.accounts.mint_authority.to_account_info(),
                    account_or_mint: ctx.accounts.mint.to_account_info(),
                },
                mint_authority_signer,
            ),
            token::spl_token::instruction::AuthorityType::MintTokens,
            None,
        )?;

        // Increment total minted count
        ctx.accounts.edition_state.total_minted = ctx
            .accounts
            .edition_state
            .total_minted
            .checked_add(1)
            .ok_or(CustomError::Overflow)?;

        Ok(())
    }

    pub fn burn_and_withdraw(ctx: Context<BurnAndWithdraw>) -> Result<()> {
        // Verify vault balance
        require_eq!(
            ctx.accounts.vault.to_account_info().lamports(),
            VAULT_AMOUNT,
            CustomError::InvalidVaultBalance
        );

        // Verify token account ownership and amount
        let token_account = token::accessor::amount(&ctx.accounts.token_account)?;
        require_eq!(token_account, 1, CustomError::InvalidTokenAmount);

        let token_owner = token::accessor::authority(&ctx.accounts.token_account)?;
        require_eq!(
            token_owner,
            ctx.accounts.owner.key(),
            CustomError::InvalidTokenAccount
        );

        // Burn NFT
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            1,
        )?;

        // Close token account
        token::close_account(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::CloseAccount {
                account: ctx.accounts.token_account.to_account_info(),
                destination: ctx.accounts.owner.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ))?;

        // Verify token account is closed by checking its lamports are 0
        require_eq!(
            ctx.accounts.token_account.lamports(),
            0,
            CustomError::TokenAccountNotClosed
        );

        // Transfer SOL back to owner
        **ctx.accounts.vault.to_account_info().lamports.borrow_mut() = 0;
        **ctx.accounts.owner.to_account_info().lamports.borrow_mut() += VAULT_AMOUNT;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeMasterEdition<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Master authority PDA
    #[account(
        seeds = ["master_authority".as_bytes(), master_mint.key().as_ref()],
        bump
    )]
    pub master_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 8, // discriminator + authority + master_mint + total_minted
        seeds = ["edition_state".as_bytes()],
        bump
    )]
    pub edition_state: Account<'info, EditionState>,

    /// CHECK: Will be initialized
    #[account(mut)]
    pub master_mint: AccountInfo<'info>,

    /// CHECK: Metadata account for master edition
    #[account(
        mut,
        seeds = ["metadata".as_bytes(), master_mint.key().as_ref(), token_program.key().as_ref()],
        bump,
        seeds::program = METADATA_PROGRAM_ID
    )]
    pub master_metadata: UncheckedAccount<'info>,

    /// CHECK: Master edition account
    #[account(
        mut,
        seeds = ["metadata".as_bytes(), master_mint.key().as_ref(), "edition".as_bytes()],
        bump,
        seeds::program = METADATA_PROGRAM_ID
    )]
    pub master_edition: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintPNFT<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 32, // discriminator + owner + mint
        seeds = ["vault".as_bytes(), mint.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, TokenVault>,

    #[account(
        mut,
        seeds = ["edition_state".as_bytes()],
        bump,
    )]
    pub edition_state: Account<'info, EditionState>,

    /// CHECK: Master edition verified by seeds
    #[account(
        mut,
        seeds = ["metadata".as_bytes(), edition_state.master_mint.as_ref(), "edition".as_bytes()],
        bump,
        seeds::program = METADATA_PROGRAM_ID
    )]
    pub master_edition: UncheckedAccount<'info>,

    /// CHECK: Metadata account created via CPI
    #[account(
        mut,
        seeds = ["metadata".as_bytes(), mint.key().as_ref(), token_program.key().as_ref()],
        bump,
        seeds::program = METADATA_PROGRAM_ID
    )]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Edition account created via CPI
    #[account(
        mut,
        seeds = ["metadata".as_bytes(), mint.key().as_ref(), "edition".as_bytes()],
        bump,
        seeds::program = METADATA_PROGRAM_ID
    )]
    pub edition_marker: UncheckedAccount<'info>,

    /// CHECK: Will be initialized
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    /// CHECK: Mint authority PDA
    #[account(
        seeds = ["mint_authority".as_bytes(), mint.key().as_ref()],
        bump
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: Will be initialized
    #[account(mut)]
    pub token_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BurnAndWithdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = ["vault".as_bytes(), mint.key().as_ref()],
        bump,
        has_one = owner,
        has_one = mint,
        close = owner
    )]
    pub vault: Account<'info, TokenVault>,

    /// CHECK: Metadata verified by seeds
    #[account(
        mut,
        seeds = ["metadata".as_bytes(), mint.key().as_ref(), token_program.key().as_ref()],
        bump,
        seeds::program = METADATA_PROGRAM_ID
    )]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Token account validated in instruction
    #[account(mut)]
    pub token_account: AccountInfo<'info>,

    /// CHECK: Mint validated by vault constraint
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct EditionState {
    pub authority: Pubkey,   // Edition authority
    pub master_mint: Pubkey, // Master edition mint
    pub total_minted: u64,   // Total NFTs minted in this edition
}

#[account]
#[derive(Default)]
pub struct TokenVault {
    pub owner: Pubkey, // 32
    pub mint: Pubkey,  // 32
}
