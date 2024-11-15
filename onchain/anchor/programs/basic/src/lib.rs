use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Token},
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

pub const METADATA_SIZE: usize = 679; // Fixed size for Metadata account

#[derive(Accounts)]
#[instruction(uri: String)]
pub struct InitializeMasterEdition<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Master authority PDA
    #[account(
        seeds = ["master_authority".as_bytes(), master_mint.key().as_ref()],
        bump,
    )]
    pub master_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = authority,
        space = EditionState::SPACE,
        seeds = ["edition_state".as_bytes(), master_mint.key().as_ref()],
        bump
    )]
    pub edition_state: Account<'info, EditionState>,

    #[account(
        init,                            
        payer = authority,              
        seeds = ["master_mint".as_bytes()],
        bump,
        space = 82,
    )]
    /// CHECK: Will be initialized
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
        space = TokenVault::SPACE,
        seeds = ["vault".as_bytes(), mint.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, TokenVault>,

    #[account(
        mut,
        seeds = ["edition_state".as_bytes(), edition_state.master_mint.as_ref()], 
        bump,
        constraint = edition_state.is_initialized @ CustomError::NotInitialized,
        constraint = edition_state.total_minted < MAX_SUPPLY @ CustomError::MaxSupplyReached,
    )]
    pub edition_state: Account<'info, EditionState>,

    /// CHECK: Master authority PDA with explicit derivation check
    #[account(
        seeds = ["master_authority".as_bytes(), edition_state.master_mint.as_ref()],
        bump,
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
    #[account(
        mut, 
        signer,
        constraint = server_authority.key() == edition_state.authority @ CustomError::UnauthorizedUpdate
    )]
    /// CHECK: Server authority
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

    #[account(
        mut, 
        seeds = ["edition_state".as_bytes(), edition_state.master_mint.as_ref()], 
        bump
    )]
    pub edition_state: Account<'info, EditionState>,
}

#[derive(Accounts)]
pub struct BurnAndWithdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut, seeds = ["edition_state".as_bytes(), edition_state.master_mint.key().as_ref()], bump)]
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
        metadata.collection.map(|c| c.key == edition_state.master_mint && c.verified).ok_or(CustomError::InvalidCollection)?
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

impl EditionState {
    pub const SPACE: usize = 8 + // discriminator
                            1 + // is_initialized 
                            32 + // authority
                            32 + // master_mint
                            8; // total_minted
}

#[account]
#[derive(Default)]
pub struct TokenVault {
    pub mint: Pubkey,
}

impl TokenVault {
    pub const SPACE: usize = 8 + 32; // discriminator + mint

    pub fn get_required_balance(&self, rent: &Rent) -> Result<u64> {
        Ok(VAULT_AMOUNT + rent.minimum_balance(Self::SPACE))
    }

    pub fn validate_balance(&self, account_info: &AccountInfo, rent: &Rent) -> Result<()> {
        require_eq!(
            account_info.lamports(),
            self.get_required_balance(rent)?, // Updated to use instance method
            CustomError::InvalidVaultBalance
        );
        Ok(())
    }
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

        let mint_rent = ctx.accounts.rent.minimum_balance(82);

        let required_balance_for_mint = mint_rent
            .checked_add(ctx.accounts.master_mint.to_account_info().lamports())
            .unwrap();

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.authority.to_account_info(),
                    to: ctx.accounts.master_mint.to_account_info(),
                },
            ),
            required_balance_for_mint,
        )?;

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

        let mint_rent = ctx.accounts.rent.minimum_balance(82);
        let metadata_rent = ctx.accounts.rent.minimum_balance(METADATA_SIZE);

        let required_vault_balance = ctx
            .accounts
            .vault
            .get_required_balance(&ctx.accounts.rent)?;

        // Transfer SOL to vault
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                },
            ),
            required_vault_balance + mint_rent + metadata_rent, // Include rent
        )?;

        // Initialize vault
        ctx.accounts.vault.mint = ctx.accounts.mint.key();

        // Initialize mint with PDA authority
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
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
            ),
            1,
        )?;

        // Create metadata
        let required_balance_for_metadata = metadata_rent
            .checked_add(ctx.accounts.metadata.to_account_info().lamports())
            .unwrap();

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.metadata.to_account_info(),
                },
            ),
            required_balance_for_metadata,
        )?;

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
        // Verify vault balance includes both locked amount and rent
        ctx.accounts
            .vault
            .validate_balance(&ctx.accounts.vault.to_account_info(), &ctx.accounts.rent)?;

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

        // Verify token account is closed
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
