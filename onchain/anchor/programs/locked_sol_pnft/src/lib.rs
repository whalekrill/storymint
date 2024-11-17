use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Token},
};
use mpl_token_metadata::accounts::Metadata;
use mpl_token_metadata::instructions::{
    Burn, BurnInstructionArgs, CreateMasterEditionV3, CreateMasterEditionV3InstructionArgs,
    CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs, VerifyCollection,
};
use mpl_token_metadata::types::{BurnArgs, Collection, DataV2};
use mpl_token_metadata::ID as METADATA_PROGRAM_ID;

declare_id!("3kLyy6249ZFsZyG74b6eSwuvDUVndkFM54cvK8gnietr");

#[cfg(not(feature = "mainnet"))]
pub const SERVER_AUTHORITY: Pubkey = pubkey!("EiLANmnffXVXczyimnGEKSZpzwQ4TyuQXVAviqBji8TF");

#[cfg(feature = "mainnet")]
pub const SERVER_AUTHORITY: Pubkey = pubkey!("ToDo44444444444444444444444444444444444444"); // TODO: Update with real mainnet address

pub const NAME: &str = "Locked SOL NFT";
pub const SYMBOL: &str = "LSOL";
pub const URI: &str = "https://api.locked-sol.com/metadata/initial.json";
pub const SELLER_FEE_BASIS_POINTS: u16 = 0;

const VAULT_AMOUNT: u64 = 1_000_000_000; // 1 SOL
const MAX_SUPPLY: u64 = 10_000;
pub const METADATA_SIZE: usize = 679;

#[derive(Accounts)]
pub struct InitializeMasterEdition<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Master state account
    #[account(
        init,
        payer = payer,
        space = MasterState::SPACE,
        seeds = ["master".as_bytes(), master_mint.key().as_ref()],
        bump
    )]
    pub master_state: Account<'info, MasterState>,

    /// CHECK: Master mint account
    #[account(
        init,
        payer = payer,
        space = anchor_spl::token::Mint::LEN,
        seeds = ["master_mint".as_bytes()],
        bump,
        owner = token_program.key()
    )]
    pub master_mint: AccountInfo<'info>,

    /// CHECK: Metadata account for master edition
    #[account(mut)]
    pub master_metadata: UncheckedAccount<'info>,

    /// CHECK: Master edition account
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    /// CHECK: Server authority for metadata updates
    #[account(
        mut,
        signer,
        constraint = update_authority.key() == SERVER_AUTHORITY @ CustomError::InvalidUpdateAuthority
    )]
    pub update_authority: Signer<'info>,

    /// CHECK: Token account for the server authority
    #[account(mut)]
    pub authority_token: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,

    /// CHECK: This is the Metaplex Token Metadata program
    #[account(address = METADATA_PROGRAM_ID)]
    pub token_metadata_program: AccountInfo<'info>,
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
        seeds = ["master".as_bytes(), master_state.master_mint.as_ref()], 
        bump,
        constraint = master_state.total_minted < MAX_SUPPLY @ CustomError::MaxSupplyReached,
    )]
    pub master_state: Account<'info, MasterState>,

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

    #[account(
        init,
        payer = payer,
        space = utils::MINT_SPACE,
        owner = token_program.key(),
    )]
    /// CHECK: Initialized as mint in instruction
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
        constraint = server_authority.key() == SERVER_AUTHORITY @ CustomError::UnauthorizedUpdate
    )]
    /// CHECK: Server authority against constant
    pub server_authority: AccountInfo<'info>,

    #[account(
        mut,
        seeds = ["vault".as_bytes(), mint.key().as_ref()],
        bump,
        has_one = mint,
    )]
    pub vault: Account<'info, TokenVault>,

    #[account(
        mut, 
        seeds = ["master".as_bytes(), master_state.master_mint.as_ref()], 
        bump
    )]
    pub master_state: Account<'info, MasterState>,

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
}

#[derive(Accounts)]
pub struct BurnAndWithdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut, 
        seeds = ["master".as_bytes(), master_state.master_mint.as_ref()], 
        bump
    )]
    pub master_state: Account<'info, MasterState>,

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
            metadata.collection.map(|c| c.key == master_state.master_mint && c.verified).ok_or(CustomError::InvalidCollection)?
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
pub struct MasterState {
    pub master_mint: Pubkey,
    pub total_minted: u64,
}

impl MasterState {
    pub const SPACE: usize = 8 + 32 + 8; // discriminator + master_mint + total_minted
}

#[account]
#[derive(Default)]
pub struct TokenVault {
    pub mint: Pubkey,
}

impl TokenVault {
    pub const SPACE: usize = 8 + 32; // discriminator + mint

    pub fn validate_balance(&self, account_info: &AccountInfo, rent: &Rent) -> Result<()> {
        let required_balance = VAULT_AMOUNT + rent.minimum_balance(Self::SPACE);
        require_eq!(
            account_info.lamports(),
            required_balance,
            CustomError::InvalidVaultBalance
        );
        Ok(())
    }
}

#[error_code]
pub enum CustomError {
    #[msg("Invalid vault balance")]
    InvalidVaultBalance,
    #[msg("Unauthorized metadata update")]
    UnauthorizedUpdate,
    #[msg("Token account not properly closed")]
    TokenAccountNotClosed,
    #[msg("Maximum supply reached")]
    MaxSupplyReached,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Invalid collection data")]
    InvalidCollection,
    #[msg("Arithmetic underflow")]
    Underflow,
    #[msg("Account is owned by wrong program")]
    InvalidProgramId,
    #[msg("Invalid update authority")]
    InvalidUpdateAuthority,
    #[msg("Metadata deserialization failed")]
    MetadataDeserializationError,
    #[msg("Collection verification failed")]
    CollectionVerificationError,
    #[msg("Invalid metadata data")]
    InvalidMetadata,
    #[msg("Invalid collection verification")]
    InvalidCollectionVerification,
}

#[program]
pub mod locked_sol_pnft {
    use super::*;

    pub fn initialize_master_edition(ctx: Context<InitializeMasterEdition>) -> Result<()> {
        let master_state = &mut ctx.accounts.master_state;
        master_state.master_mint = ctx.accounts.master_mint.key();
        master_state.total_minted = 0;

        // Initialize mint with server authority
        token::initialize_mint(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::InitializeMint {
                    mint: ctx.accounts.master_mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            0,
            &ctx.accounts.update_authority.key(), // Server authority as mint authority
            Some(&ctx.accounts.update_authority.key()), // And freeze authority
        )?;

        // Create ATA for server authority
        anchor_spl::associated_token::create(CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: ctx.accounts.payer.to_account_info(),
                associated_token: ctx.accounts.authority_token.to_account_info(),
                authority: ctx.accounts.update_authority.to_account_info(),
                mint: ctx.accounts.master_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
        ))?;

        // Create metadata
        let metadata_data = get_initial_metadata(None);
        create_metadata(
            &ctx.accounts.payer,
            &ctx.accounts.master_metadata,
            &ctx.accounts.master_mint,
            &ctx.accounts.update_authority,
            &ctx.accounts.system_program,
            &ctx.accounts.rent,
            metadata_data,
            &[], // No signing seeds needed
        )?;

        // Mint token directly as server authority
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.master_mint.to_account_info(),
                    to: ctx.accounts.authority_token.to_account_info(),
                    authority: ctx.accounts.update_authority.to_account_info(),
                },
            ),
            1,
        )?;

        // Create master edition
        create_master_edition_for_master(&ctx, Some(MAX_SUPPLY), &[])?;

        Ok(())
    }

    pub fn mint_pnft(ctx: Context<MintPNFT>) -> Result<()> {
        let rent_costs = utils::calculate_rent(&ctx.accounts.rent, true);
        let total_required = rent_costs.vault + rent_costs.mint + rent_costs.metadata;

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

        ctx.accounts.vault.mint = ctx.accounts.mint.key();

        let mint_key = ctx.accounts.mint.key();
        let mut bump_arr = [0u8; 1];
        let mint_authority_seeds =
            utils::get_mint_authority_seeds(&mint_key, ctx.bumps.mint_authority, &mut bump_arr);

        initialize_token_mint_for_mint(&ctx)?;
        create_associated_token(&ctx)?;
        mint_token(&ctx, &[&mint_authority_seeds])?;

        let collection = Some(Collection {
            verified: false,
            key: ctx.accounts.master_state.master_mint,
        });

        // Single metadata creation
        create_metadata(
            &ctx.accounts.payer,
            &ctx.accounts.metadata,
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.mint_authority,
            &ctx.accounts.system_program,
            &ctx.accounts.rent,
            get_initial_metadata(collection),
            &[&mint_authority_seeds],
        )?;

        create_master_edition_for_mint(&ctx, Some(0), &[&mint_authority_seeds])?;

        let master_mint_key = ctx.accounts.master_state.master_mint;
        verify_collection(
            &ctx,
            &[&[
                b"master".as_ref(),
                master_mint_key.as_ref(),
                &[ctx.bumps.master_state],
            ]],
        )?;

        ctx.accounts.master_state.total_minted = ctx
            .accounts
            .master_state
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
                        key: ctx.accounts.master_state.master_mint,
                    }),
                    uses: None,
                }),
                new_update_authority: Some(ctx.accounts.server_authority.key()),
                primary_sale_happened: None,
                is_mutable: Some(true),
            },
        );

        invoke_signed(
            &update_metadata_ix,
            &[
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.server_authority.to_account_info(),
            ],
            &[],
        )
        .map_err(Into::into)
    }

    pub fn burn_and_withdraw(ctx: Context<BurnAndWithdraw>) -> Result<()> {
        ctx.accounts
            .vault
            .validate_balance(&ctx.accounts.vault.to_account_info(), &ctx.accounts.rent)?;

        burn_nft(&ctx)?;

        token::close_account(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::CloseAccount {
                account: ctx.accounts.token_account.to_account_info(),
                destination: ctx.accounts.owner.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ))?;

        require_eq!(
            ctx.accounts.token_account.to_account_info().lamports(),
            0,
            CustomError::TokenAccountNotClosed
        );

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

        ctx.accounts.master_state.total_minted = ctx
            .accounts
            .master_state
            .total_minted
            .checked_sub(1)
            .ok_or(CustomError::Underflow)?;

        Ok(())
    }
}

mod utils {
    use super::*;

    pub const MINT_SPACE: usize = 82;

    pub struct ProgramRent {
        pub vault: u64,
        pub mint: u64,
        pub metadata: u64,
    }

    pub fn calculate_rent(rent: &Rent, include_vault_amount: bool) -> ProgramRent {
        let vault = if include_vault_amount {
            VAULT_AMOUNT + rent.minimum_balance(TokenVault::SPACE)
        } else {
            rent.minimum_balance(TokenVault::SPACE)
        };

        ProgramRent {
            vault,
            mint: rent.minimum_balance(MINT_SPACE),
            metadata: rent.minimum_balance(METADATA_SIZE),
        }
    }

    pub fn get_mint_authority_seeds<'a>(
        mint: &'a Pubkey,
        bump: u8,
        bump_arr: &'a mut [u8; 1],
    ) -> [&'a [u8]; 3] {
        const PREFIX: &[u8] = b"mint_authority";
        bump_arr[0] = bump;
        [PREFIX, mint.as_ref(), &bump_arr[..]]
    }
}

fn get_initial_metadata(collection: Option<Collection>) -> DataV2 {
    DataV2 {
        name: NAME.to_string(),
        symbol: SYMBOL.to_string(),
        uri: URI.to_string(),
        seller_fee_basis_points: SELLER_FEE_BASIS_POINTS,
        creators: None,
        collection,
        uses: None,
    }
}

fn create_metadata<'info>(
    payer: &AccountInfo<'info>,
    metadata: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    mint_authority: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    rent: &Sysvar<'info, Rent>,
    metadata_data: DataV2,
    signing_seeds: &[&[&[u8]]],
) -> Result<()> {
    let create_metadata_ix = CreateMetadataAccountV3 {
        metadata: metadata.key(),
        mint: mint.key(),
        mint_authority: mint_authority.key(),
        payer: payer.key(),
        update_authority: (SERVER_AUTHORITY, true),
        system_program: system_program.key(),
        rent: None,
    }
    .instruction(CreateMetadataAccountV3InstructionArgs {
        data: metadata_data,
        is_mutable: true,
        collection_details: None,
    });

    invoke_signed(
        &create_metadata_ix,
        &[
            metadata.to_account_info(),
            mint.to_account_info(),
            mint_authority.to_account_info(),
            payer.to_account_info(),
            system_program.to_account_info(),
            rent.to_account_info(),
        ],
        signing_seeds,
    )
    .map_err(Into::into)
}

fn create_associated_token(ctx: &Context<MintPNFT>) -> Result<()> {
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
    ))
}

fn mint_token<'info>(ctx: &Context<MintPNFT>, signer_seeds: &[&[&[u8]]]) -> Result<()> {
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer_seeds,
        ),
        1,
    )
}

fn create_master_edition_for_master(
    ctx: &Context<InitializeMasterEdition>,
    max_supply: Option<u64>,
    signing_seeds: &[&[&[u8]]],
) -> Result<()> {
    let create_master_edition_ix = CreateMasterEditionV3 {
        edition: ctx.accounts.master_edition.key(),
        mint: ctx.accounts.master_mint.key(),
        update_authority: ctx.accounts.master_state.key(),
        mint_authority: ctx.accounts.master_state.key(),
        metadata: ctx.accounts.master_metadata.key(),
        payer: ctx.accounts.payer.key(),
        token_program: ctx.accounts.token_program.key(),
        system_program: ctx.accounts.system_program.key(),
        rent: None,
    }
    .instruction(CreateMasterEditionV3InstructionArgs { max_supply });

    invoke_signed(
        &create_master_edition_ix,
        &[
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.master_mint.to_account_info(),
            ctx.accounts.master_state.to_account_info(),
            ctx.accounts.master_state.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.master_metadata.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
        ],
        signing_seeds,
    )
    .map_err(Into::into)
}

fn create_master_edition_for_mint(
    ctx: &Context<MintPNFT>,
    max_supply: Option<u64>,
    signing_seeds: &[&[&[u8]]],
) -> Result<()> {
    let create_master_edition_ix = CreateMasterEditionV3 {
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
    .instruction(CreateMasterEditionV3InstructionArgs { max_supply });

    invoke_signed(
        &create_master_edition_ix,
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
        signing_seeds,
    )
    .map_err(Into::into)
}

fn initialize_token_mint_for_mint(ctx: &Context<MintPNFT>) -> Result<()> {
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
    )
}

fn verify_collection(ctx: &Context<MintPNFT>, signing_seeds: &[&[&[u8]]]) -> Result<()> {
    let verify_collection_ix = VerifyCollection {
        collection_authority: ctx.accounts.master_state.key(),
        payer: ctx.accounts.payer.key(),
        metadata: ctx.accounts.metadata.key(),
        collection_mint: ctx.accounts.master_state.master_mint,
        collection: ctx.accounts.collection_metadata.key(),
        collection_master_edition_account: ctx.accounts.collection_master_edition.key(),
        collection_authority_record: None,
    }
    .instruction();

    invoke_signed(
        &verify_collection_ix,
        &[
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.master_state.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.collection_metadata.to_account_info(),
            ctx.accounts.collection_master_edition.to_account_info(),
        ],
        signing_seeds,
    )
    .map_err(Into::into)
}

fn burn_nft(ctx: &Context<BurnAndWithdraw>) -> Result<()> {
    // Burn token
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

    invoke_signed(
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
    )
    .map_err(Into::into)
}
