use crate::ID as PROGRAM_ID;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Token, TokenAccount},
};
use mpl_token_metadata::instructions::{
    Burn, BurnInstructionArgs, CreateMasterEditionV3, CreateMasterEditionV3InstructionArgs,
    CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs, VerifyCollection,
};
use mpl_token_metadata::types::{BurnArgs, Collection, DataV2};
use mpl_token_metadata::{accounts::Metadata, ID as METADATA_PROGRAM_ID};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const VAULT_AMOUNT: u64 = 1_000_000_000; // 1 SOL in lamports
const MAX_SUPPLY: u64 = 10_000; // Max NFTs in master edition

#[derive(Accounts)]
#[instruction(uri: String)]
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
        space = 8 + 1 + 32 + 32 + 8,
        seeds = ["edition_state".as_bytes(), master_mint.key().as_ref()],
        bump
    )]
    pub edition_state: Account<'info, EditionState>,

    /// CHECK: Will be initialized
    #[account(
        init,                            
        payer = authority,              
        seeds = ["master_mint".as_bytes()],
        bump,
        space = 82,
    )]
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

    /// CHECK: Verified statically using constraint
    #[account(address = METADATA_PROGRAM_ID @ CustomError::InvalidProgramId)]
    pub token_metadata_program: UncheckedAccount<'info>,

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
        space = 8 + 32 + 32,
        seeds = ["vault".as_bytes(), mint.key().as_ref()],
        bump,
        owner = PROGRAM_ID
    )]
    pub vault: Account<'info, TokenVault>,

    #[account(
        mut,
        seeds = ["edition_state".as_bytes()],
        bump,
        constraint = edition_state.is_initialized @ CustomError::NotInitialized,
        owner = PROGRAM_ID @ CustomError::InvalidProgramId
    )]
    pub edition_state: Account<'info, EditionState>,

    /// CHECK: Master authority PDA with explicit derivation check
    #[account(
        seeds = ["master_authority".as_bytes(), edition_state.master_mint.as_ref()],
        bump,
        owner = PROGRAM_ID @ CustomError::InvalidProgramId
    )]
    pub master_authority: UncheckedAccount<'info>,

    /// CHECK: Collection metadata with explicit program check
    #[account(
        mut,
        owner = mpl_token_metadata::ID @ CustomError::InvalidProgramId
    )]
    pub collection_metadata: UncheckedAccount<'info>,

    /// CHECK: Collection master edition with explicit program check
    #[account(
        owner = mpl_token_metadata::ID @ CustomError::InvalidProgramId
    )]
    pub collection_master_edition: UncheckedAccount<'info>,

    /// CHECK: Will be initialized as metadata
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Edition marker account with explicit derivation
    #[account(
        mut,
        seeds = ["metadata".as_bytes(), mint.key().as_ref(), "edition".as_bytes()],
        bump,
        seeds::program = mpl_token_metadata::ID
    )]
    pub edition_marker: UncheckedAccount<'info>,

    /// CHECK: Mint account to be initialized
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    /// CHECK: Mint authority PDA with explicit derivation
    #[account(
        seeds = ["mint_authority".as_bytes(), mint.key().as_ref()],
        bump,
        owner = PROGRAM_ID @ CustomError::InvalidProgramId
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: Server authority for metadata updates
    pub server_authority: AccountInfo<'info>,

    /// CHECK: Token account to be initialized
    #[account(mut)]
    pub token_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateMetadata<'info> {
    /// CHECK: Server authority
    #[account(mut, signer, constraint = server_authority.key() == edition_state.authority @ CustomError::UnauthorizedUpdate)]
    pub server_authority: AccountInfo<'info>,

    #[account(
        mut,
        seeds = ["vault".as_bytes(), mint.key().as_ref()],
        bump,
        has_one = mint,
    )]
    pub vault: Account<'info, TokenVault>,

    /// CHECK: Metadata account with explicit program check
    #[account(
        mut,
        seeds = ["metadata".as_bytes(), mint.key().as_ref(), token_program.key().as_ref()],
        bump,
        seeds::program = METADATA_PROGRAM_ID
    )]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Mint account verified through constraints
    pub mint: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    // Removed sysvar_instructions (unused)
    #[account(mut, seeds = ["edition_state".as_bytes(), edition_state.master_mint.as_ref()], bump)]
    pub edition_state: Account<'info, EditionState>,
}

#[derive(Accounts)]
pub struct BurnAndWithdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub edition_state: Account<'info, EditionState>,

    #[account(
        mut,
        seeds = ["vault".as_bytes(), mint.key().as_ref()],
        bump,
        has_one = mint,
        close = owner
    )]
    pub vault: Account<'info, TokenVault>,

    /// CHECK: Token Metadata Program
    #[account(address = METADATA_PROGRAM_ID @ CustomError::InvalidProgramId)]
    pub token_metadata_program: AccountInfo<'info>,

    /// CHECK: Metadata account verified by seeds and collection
    #[account(
        mut,
        seeds = ["metadata".as_bytes(), mint.key().as_ref(), token_program.key().as_ref()],
        bump,
        seeds::program = METADATA_PROGRAM_ID,
        constraint = {
            let metadata = Metadata::safe_deserialize(&metadata.data.borrow())?;
            metadata.collection.as_ref()
                .and_then(|c| if c.verified {
                    Some(c.key == edition_state.master_mint)
                } else {
                    None
                })
                .ok_or(CustomError::InvalidCollection)?
        }
    )]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Token account
    #[account(mut)]
    pub token_account: AccountInfo<'info>,

    /// CHECK: Mint account
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    /// CHECK: Required by token metadata program
    pub sysvar_instructions: AccountInfo<'info>,

    /// CHECK: Edition marker account for burning
    #[account(
        mut,
        seeds = ["metadata".as_bytes(), mint.key().as_ref(), "edition".as_bytes()],
        bump,
        seeds::program = METADATA_PROGRAM_ID
    )]
    pub edition_marker: UncheckedAccount<'info>,
}

#[account]
pub struct EditionState {
    pub is_initialized: bool,
    pub authority: Pubkey,
    pub master_mint: Pubkey,
    pub total_minted: u64,
}

#[account]
#[derive(Default)]
pub struct TokenVault {
    pub mint: Pubkey,
}

#[error_code]
pub enum CustomError {
    InvalidTokenAmount,
    InvalidVaultBalance,
    UnauthorizedUpdate,
    InvalidTokenAccount,
    TokenAccountNotClosed,
    MaxSupplyReached,
    Overflow,
    NotInitialized,
    UnauthorizedAdmin,
    InsufficientBalance,
    BalanceOverflow,
    InvalidCollection,
    Underflow,
    #[msg("Account is owned by wrong program")]
    InvalidProgramId,
    #[msg("Account is not initialized")]
    UninitializedAccount,
    #[msg("Account is already initialized")]
    AccountAlreadyInitialized,
    #[msg("Invalid account derivation")]
    InvalidDerivation,
    #[msg("Invalid update authority")]
    InvalidUpdateAuthority,
    #[msg("Math overflow")]
    MathOverflow,
}

pub fn validate_pda_derivation(pda: &Pubkey, seeds: &[&[u8]], bump: u8) -> Result<()> {
    let (derived_pda, derived_bump) = Pubkey::find_program_address(seeds, &crate::ID);
    require_keys_eq!(*pda, derived_pda, CustomError::InvalidDerivation);
    require_eq!(bump, derived_bump, CustomError::InvalidDerivation);
    Ok(())
}

pub mod vault_utils {
    use super::*;

    const VAULT_SIZE: usize = 8 + 32; // discriminator + mint

    pub fn get_vault_rent_exempt_balance(rent: &Rent) -> u64 {
        rent.minimum_balance(VAULT_SIZE)
    }

    pub fn get_required_vault_balance(rent: &Rent) -> u64 {
        VAULT_AMOUNT
            .checked_add(get_vault_rent_exempt_balance(rent))
            .expect("Vault balance overflow")
    }

    pub fn validate_vault_balance(vault: &AccountInfo, rent: &Rent) -> Result<()> {
        let required_balance = get_required_vault_balance(rent);
        let current_balance = vault.lamports();

        require_eq!(
            current_balance,
            required_balance,
            CustomError::InvalidVaultBalance
        );

        Ok(())
    }

    pub fn validate_account_balances<'info>(
        payer: &AccountInfo<'info>,
        vault: &AccountInfo<'info>,
        rent: &Rent,
        is_initialization: bool,
    ) -> Result<()> {
        let required_balance = get_required_vault_balance(rent);

        // During initialization, check payer has enough funds
        if is_initialization {
            require_gte!(
                payer.lamports(),
                required_balance,
                CustomError::InsufficientBalance
            );
        }
        // For existing vaults, validate current balance
        else {
            validate_vault_balance(vault, rent)?;
        }

        Ok(())
    }

    pub fn validate_withdrawal_balance<'info>(
        vault: &AccountInfo<'info>,
        owner: &AccountInfo<'info>,
        rent: &Rent,
    ) -> Result<()> {
        // Ensure vault has exactly the required balance
        validate_vault_balance(vault, rent)?;

        // Calculate expected owner balance after withdrawal
        let withdrawal_amount = VAULT_AMOUNT
            .checked_add(get_vault_rent_exempt_balance(rent))
            .ok_or(CustomError::Overflow)?;

        // Ensure owner can receive the funds (optional check)
        require_gte!(
            owner
                .lamports()
                .checked_add(withdrawal_amount)
                .ok_or(CustomError::Overflow)?,
            withdrawal_amount,
            CustomError::BalanceOverflow
        );

        Ok(())
    }
}

#[program]
pub mod locked_sol_pnft {
    use super::*;

    pub fn initialize_master_edition(
        ctx: Context<InitializeMasterEdition>,
        uri: String,
    ) -> Result<()> {
        // Initialize collection state
        ctx.accounts.edition_state.is_initialized = true;
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
            rent: None,
        }
        .instruction(CreateMetadataAccountV3InstructionArgs {
            data: DataV2 {
                name: "Locked SOL NFT".to_string(),
                symbol: "LSOL".to_string(),
                uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            is_mutable: true,
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

        // Create master edition
        let create_master_edition_ix = CreateMasterEditionV3 {
            edition: ctx.accounts.master_edition.key(),
            mint: ctx.accounts.master_mint.key(),
            update_authority: ctx.accounts.master_authority.key(),
            mint_authority: ctx.accounts.master_authority.key(),
            metadata: ctx.accounts.master_metadata.key(),
            payer: ctx.accounts.authority.key(),
            token_program: ctx.accounts.token_program.key(),
            system_program: ctx.accounts.system_program.key(),
            rent: None,
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
        // Check edition state is initialized
        require!(
            ctx.accounts.edition_state.is_initialized,
            CustomError::NotInitialized
        );

        // Check we haven't exceeded max supply
        require!(
            ctx.accounts.edition_state.total_minted < MAX_SUPPLY,
            CustomError::MaxSupplyReached
        );

        vault_utils::validate_account_balances(
            &ctx.accounts.payer.to_account_info(),
            &ctx.accounts.vault.to_account_info(),
            &ctx.accounts.rent,
            true,
        )?;

        // Transfer SOL to vault
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                },
            ),
            vault_utils::get_required_vault_balance(&ctx.accounts.rent),
        )?;

        // Initialize vault
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

        // Create metadata
        let create_metadata_ix = CreateMetadataAccountV3 {
            metadata: ctx.accounts.metadata.key(),
            mint: ctx.accounts.mint.key(),
            mint_authority: ctx.accounts.mint_authority.key(),
            payer: ctx.accounts.payer.key(),
            update_authority: (ctx.accounts.master_authority.key(), true),
            system_program: ctx.accounts.system_program.key(),
            rent: None,
        }
        .instruction(CreateMetadataAccountV3InstructionArgs {
            data: DataV2 {
                name: "Locked SOL NFT".to_string(),
                symbol: "LSOL".to_string(),
                uri: metadata_uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: Some(Collection {
                    verified: false,
                    key: ctx.accounts.edition_state.master_mint,
                }),
                uses: None,
            },
            is_mutable: true,
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
            &[&[
                b"mint_authority".as_ref(),
                ctx.accounts.mint.key().as_ref(),
                &[ctx.bumps.mint_authority],
            ]],
        )?;

        // Create edition
        let master_edition_ix = CreateMasterEditionV3 {
            edition: ctx.accounts.edition_marker.key(),
            mint: ctx.accounts.mint.key(),
            update_authority: ctx.accounts.mint_authority.key(),
            mint_authority: ctx.accounts.mint_authority.key(),
            metadata: ctx.accounts.metadata.key(),
            payer: ctx.accounts.payer.key(),
            token_program: ctx.accounts.token_program.key(),
            system_program: ctx.accounts.system_program.key(),
            rent: None,
        }
        .instruction(CreateMasterEditionV3InstructionArgs {
            max_supply: Some(0), // No editions allowed for this NFT
        });

        anchor_lang::solana_program::program::invoke_signed(
            &master_edition_ix,
            &[
                ctx.accounts.edition_marker.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
            &[&[
                b"mint_authority".as_ref(),
                ctx.accounts.mint.key().as_ref(),
                &[ctx.bumps.mint_authority],
            ]],
        )?;

        // Verify collection
        let verify_collection_ix = VerifyCollection {
            collection_authority: ctx.accounts.master_authority.key(),
            payer: ctx.accounts.payer.key(),
            metadata: ctx.accounts.metadata.key(),
            collection_mint: ctx.accounts.edition_state.master_mint,
            collection: ctx.accounts.collection_metadata.key(),
            collection_master_edition_account: ctx.accounts.collection_master_edition.key(),
            collection_authority_record: None,
        }
        .instruction();

        anchor_lang::solana_program::program::invoke_signed(
            &verify_collection_ix,
            &[
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.master_authority.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.collection_metadata.to_account_info(),
                ctx.accounts.collection_master_edition.to_account_info(),
            ],
            &[&[
                b"master_authority".as_ref(),
                ctx.accounts.edition_state.master_mint.as_ref(),
                &[ctx.bumps.master_authority],
            ]],
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

    pub fn update_metadata(
        ctx: Context<UpdateMetadata>,
        new_uri: String,
        new_name: Option<String>,
    ) -> Result<()> {
        require!(
            ctx.accounts.server_authority.is_signer,
            CustomError::InvalidUpdateAuthority
        );

        let update_metadata_ix = mpl_token_metadata::instructions::UpdateMetadataAccountV2 {
            metadata: ctx.accounts.metadata.key(),
            update_authority: ctx.accounts.server_authority.key(),
        }
        .instruction(
            mpl_token_metadata::instructions::UpdateMetadataAccountV2InstructionArgs {
                data: Some(DataV2 {
                    name: new_name.unwrap_or("Locked SOL NFT".to_string()),
                    symbol: "LSOL".to_string(),
                    uri: new_uri,
                    seller_fee_basis_points: 0,
                    creators: None,
                    collection: Some(Collection {
                        verified: true,
                        key: ctx.accounts.edition_state.master_mint,
                    }),
                    uses: None,
                }),
                new_update_authority: Some(ctx.accounts.server_authority.key()),
                primary_sale_happened: None,
                is_mutable: Some(true),
            },
        );

        anchor_lang::solana_program::program::invoke_signed(
            &update_metadata_ix,
            &[
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.server_authority.to_account_info(),
            ],
            &[],
        )?;

        Ok(())
    }

    pub fn burn_and_withdraw(ctx: Context<BurnAndWithdraw>) -> Result<()> {
        let token_account_data = ctx.accounts.token_account.try_borrow_data()?;
        let token_account = TokenAccount::try_deserialize(&mut &token_account_data[..])?;

        require_eq!(token_account.mint, ctx.accounts.mint.key());
        require_eq!(token_account.owner, ctx.accounts.owner.key());
        require_eq!(token_account.amount, 1);

        // Verify vault balance includes both locked amount and rent
        let rent_exempt = ctx.accounts.rent.minimum_balance(
            8 + 32 + 32, // TokenVault size
        );
        let expected_balance = VAULT_AMOUNT
            .checked_add(rent_exempt)
            .ok_or(CustomError::Overflow)?;

        vault_utils::validate_withdrawal_balance(
            &ctx.accounts.vault.to_account_info(),
            &ctx.accounts.owner.to_account_info(),
            &ctx.accounts.rent,
        )?;

        require_eq!(
            ctx.accounts.vault.to_account_info().lamports(),
            expected_balance,
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

        // Close metadata account
        let burn_ix = Burn {
            authority: ctx.accounts.owner.key(),
            collection_metadata: None,
            metadata: ctx.accounts.metadata.key(),
            edition: None,
            mint: ctx.accounts.mint.key(),
            token: ctx.accounts.token_account.key(),
            master_edition: None,
            master_edition_mint: None,
            master_edition_token: None,
            edition_marker: Some(ctx.accounts.edition_marker.key()),
            token_record: None,
            system_program: ctx.accounts.system_program.key(),
            sysvar_instructions: ctx.accounts.sysvar_instructions.key(),
            spl_token_program: ctx.accounts.token_program.key(),
        }
        .instruction(BurnInstructionArgs {
            burn_args: BurnArgs::V1 { amount: 1 },
        });

        anchor_lang::solana_program::program::invoke_signed(
            &burn_ix,
            &[
                ctx.accounts.owner.to_account_info(),
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.token_account.to_account_info(),
                ctx.accounts.edition_marker.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.sysvar_instructions.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
            &[],
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

        // Verify token account is closed - checking lamports on AccountInfo
        require_eq!(
            ctx.accounts.token_account.to_account_info().lamports(),
            0,
            CustomError::TokenAccountNotClosed
        );

        // Transfer exact VAULT_AMOUNT back to owner
        // When closing the vault, the rent will be returned to owner
        // due to close = owner constraint in the account validation
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.owner.to_account_info(),
                },
            ),
            VAULT_AMOUNT,
        )?;

        // Decrement total_minted counter
        ctx.accounts.edition_state.total_minted = ctx
            .accounts
            .edition_state
            .total_minted
            .checked_sub(1)
            .ok_or(CustomError::Underflow)?;

        Ok(())
    }
}
