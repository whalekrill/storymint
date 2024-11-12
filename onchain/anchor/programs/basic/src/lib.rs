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
use mpl_token_metadata::types::DataV2;
use mpl_token_metadata::ID as METADATA_PROGRAM_ID;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[error_code]
pub enum CustomError {
    InvalidTokenAmount,
    InvalidVaultBalance,
    InvalidOwner,
    InvalidTokenAccountOwner,
    AlreadyWithdrawn,
    InvalidMetadataUri,
    InsufficientBalance,
    InvalidProgramOwner,
    InvalidMintAccount,
    InvalidTokenAccount,
    InvalidMetadataAccount,
}

#[event]
pub struct MintPNFTEvent {
    owner: Pubkey,
    mint: Pubkey,
    metadata_uri: String,
    vault: Pubkey,
    timestamp: i64,
}

#[event]
pub struct BurnAndWithdrawEvent {
    owner: Pubkey,
    mint: Pubkey,
    vault: Pubkey,
    amount: u64,
    timestamp: i64,
}

const VAULT_AMOUNT: u64 = 1_000_000_000; // 1 SOL in lamports

#[program]
pub mod locked_sol_pnft {
    use super::*;

    pub fn mint_pnft(ctx: Context<MintPNFT>, metadata_uri: String) -> Result<()> {
        // Check payer has sufficient balance
        require_gte!(
            ctx.accounts.payer.lamports(),
            VAULT_AMOUNT + 10_000_000, // Amount + rent + gas estimate
            CustomError::InsufficientBalance
        );

        // Verify program ownership of accounts
        require_eq!(
            ctx.accounts.mint.owner,
            &system_program::ID,
            CustomError::InvalidProgramOwner
        );

        // Transfer SOL to the vault
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

        // Initialize vault account data with version control
        ctx.accounts.vault.owner = ctx.accounts.payer.key();
        ctx.accounts.vault.amount = VAULT_AMOUNT;
        ctx.accounts.vault.withdrawn = false;
        ctx.accounts.vault.version = 1;
        ctx.accounts.vault.mint = ctx.accounts.mint.key();

        // Initialize mint with unique PDA seeds
        let mint_key = ctx.accounts.mint.key();
        let mint_authority_seeds = &[
            b"mint_authority".as_ref(),
            mint_key.as_ref(),
            &[ctx.bumps.mint_authority],
        ];
        let mint_authority_signer = &[&mint_authority_seeds[..]];

        let rent = Rent::get()?;
        let mint_rent = rent.minimum_balance(82);

        anchor_lang::system_program::create_account(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::CreateAccount {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.mint.to_account_info(),
                },
            ),
            mint_rent,
            82,
            &token::ID,
        )?;

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

        // Create and verify associated token account
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

        // Verify token account
        require_eq!(
            *ctx.accounts.token_account.owner,
            token::ID,
            CustomError::InvalidTokenAccount
        );

        // Mint token
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

        // Create metadata with additional checks
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
                uri: metadata_uri.clone(),
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
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
            mint_authority_signer,
        )?;

        // Create master edition with additional validation
        let create_master_edition_ix = CreateMasterEditionV3 {
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
            max_supply: Some(0),
        });

        anchor_lang::solana_program::program::invoke_signed(
            &create_master_edition_ix,
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

        // Revoke authorities
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

        token::set_authority(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::SetAuthority {
                    current_authority: ctx.accounts.mint_authority.to_account_info(),
                    account_or_mint: ctx.accounts.mint.to_account_info(),
                },
                mint_authority_signer,
            ),
            token::spl_token::instruction::AuthorityType::FreezeAccount,
            None,
        )?;

        emit!(MintPNFTEvent {
            owner: ctx.accounts.payer.key(),
            mint: ctx.accounts.mint.key(),
            metadata_uri,
            vault: ctx.accounts.vault.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn burn_and_withdraw(ctx: Context<BurnAndWithdraw>) -> Result<()> {
        // Additional validation checks
        require_eq!(
            ctx.accounts.mint.owner,
            &token::ID,
            CustomError::InvalidProgramOwner
        );

        require_eq!(
            *ctx.accounts.token_account.owner,
            token::ID,
            CustomError::InvalidTokenAccount
        );

        // Verify token amount and owner
        let token_amount = token::accessor::amount(&ctx.accounts.token_account)?;
        require_eq!(token_amount, 1, CustomError::InvalidTokenAmount);

        let token_owner = token::accessor::authority(&ctx.accounts.token_account)?;
        require_eq!(
            token_owner,
            ctx.accounts.owner.key(),
            CustomError::InvalidTokenAccountOwner
        );

        // Verify vault state
        require_eq!(
            ctx.accounts.vault.mint,
            ctx.accounts.mint.key(),
            CustomError::InvalidMintAccount
        );

        require_eq!(
            ctx.accounts.vault.version,
            1,
            CustomError::InvalidVaultBalance
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

        // Transfer SOL with additional checks
        let vault_balance = ctx.accounts.vault.amount;
        require_eq!(
            vault_balance,
            VAULT_AMOUNT,
            CustomError::InvalidVaultBalance
        );

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.owner.to_account_info(),
                },
            ),
            vault_balance,
        )?;

        // Update vault state
        ctx.accounts.vault.withdrawn = true;
        ctx.accounts.vault.amount = 0;
        ctx.accounts.vault.version = 2;

        emit!(BurnAndWithdrawEvent {
            owner: ctx.accounts.owner.key(),
            mint: ctx.accounts.mint.key(),
            vault: ctx.accounts.vault.key(),
            amount: vault_balance,
            timestamp: Clock::get()?.unix_timestamp,
        });

        // Close vault and return rent
        let vault_starting_lamports = ctx.accounts.vault.to_account_info().lamports();
        **ctx.accounts.vault.to_account_info().lamports.borrow_mut() = 0;
        **ctx.accounts.owner.to_account_info().lamports.borrow_mut() += vault_starting_lamports;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct MintPNFT<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 8 + 1 + 1 + 32, // Added space for version and mint
        seeds = ["vault".as_bytes(), mint.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, TokenVault>,

    /// CHECK: Metadata account created via CPI
    #[account(
        mut,
        seeds = ["metadata".as_bytes(), mint.key().as_ref(), token_program.key().as_ref()],
        bump,
        seeds::program = METADATA_PROGRAM_ID
    )]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Master Edition account created via CPI
    #[account(
        mut,
        seeds = ["metadata".as_bytes(), mint.key().as_ref(), "edition".as_bytes()],
        bump,
        seeds::program = METADATA_PROGRAM_ID
    )]
    pub master_edition: UncheckedAccount<'info>,

    /// CHECK: Will be initialized in the instruction
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    /// CHECK: Mint authority PDA
    #[account(
        seeds = ["mint_authority".as_bytes(), mint.key().as_ref()],
        bump
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: Will be initialized in the instruction
    #[account(
        mut,
        constraint = token_account.owner == &system_program::ID @ CustomError::InvalidTokenAccount
    )]
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
        constraint = !vault.withdrawn @ CustomError::AlreadyWithdrawn,
        constraint = vault.amount == VAULT_AMOUNT @ CustomError::InvalidVaultBalance,
        constraint = vault.version == 1 @ CustomError::InvalidVaultBalance,
        close = owner
    )]
    pub vault: Account<'info, TokenVault>,

    /// CHECK: Metadata account validated by seeds
    #[account(
        mut,
        seeds = ["metadata".as_bytes(), mint.key().as_ref(), token_program.key().as_ref()],
        bump,
        seeds::program = METADATA_PROGRAM_ID
    )]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Token account validated in instruction
    #[account(
        mut,
        constraint = token_account.owner == &token::ID @ CustomError::InvalidTokenAccount
    )]
    pub token_account: AccountInfo<'info>,

    /// CHECK: Mint account validated in instruction
    #[account(
        mut,
        constraint = mint.owner == &token::ID @ CustomError::InvalidProgramOwner
    )]
    pub mint: AccountInfo<'info>,

    /// CHECK: Account validated by address
    #[account(address = METADATA_PROGRAM_ID)]
    pub token_metadata_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Default)]
pub struct TokenVault {
    pub owner: Pubkey,   // 32
    pub amount: u64,     // 8
    pub withdrawn: bool, // 1
    pub version: u8,     // 1
    pub mint: Pubkey,    // 32
}
