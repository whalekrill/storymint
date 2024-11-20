use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::{invoke, invoke_signed};
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::{self, AssociatedToken, Create},
    token::{self, Token},
};

// MPL imports
use mpl_token_metadata::{
    accounts::Metadata,
    instructions::{
        BurnBuilder, CreateMasterEditionV3Builder, CreateMetadataAccountV3Builder,
        VerifyCollectionBuilder,
    },
    types::{Collection, DataV2},
    ID as METADATA_PROGRAM_ID,
};

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

    #[account(
        init,
        payer = payer,
        space = MasterState::SPACE,
        seeds = ["master".as_bytes(), master_mint.key().as_ref()],
        bump
    )]
    pub master_state: Account<'info, MasterState>,

    /// CHECK: Mint account with explicit program check
    #[account(
        init,
        payer = payer,
        space = anchor_spl::token::Mint::LEN,
        seeds = ["master_mint".as_bytes()],
        bump,
        owner = token_program.key()
    )]
    pub master_mint: AccountInfo<'info>,

    /// CHECK: Metadata account with explicit program check
    #[account(mut)]
    pub master_metadata: UncheckedAccount<'info>,

    /// CHECK: Master edition account with explicit program check
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    #[account(
        mut,
        signer,
        constraint = update_authority.key() == SERVER_AUTHORITY @ CustomError::InvalidUpdateAuthority
    )]
    pub update_authority: Signer<'info>,

    /// CHECK: Token account for the update authority
    #[account(mut)]
    pub update_authority_token: AccountInfo<'info>,

    /// CHECK: Collection authority record PDA

    #[account(mut)]
    pub collection_authority_record: UncheckedAccount<'info>,

    /// CHECK: Delegate authority from master state  
    #[account(
        mut,
        seeds = ["collection_delegate".as_bytes(), master_mint.key().as_ref()],
        bump,
    )]
    pub delegate_authority: UncheckedAccount<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub rent: Sysvar<'info, Rent>,

    /// CHECK: Required by token metadata program
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
        has_one = master_mint @ CustomError::InvalidCollection
    )]
    pub master_state: Account<'info, MasterState>,

    /// CHECK: Collection mint account required by Metaplex verify_collection
    #[account(
        constraint = master_mint.owner == &token::ID @ CustomError::InvalidProgramId
    )]
    pub master_mint: AccountInfo<'info>,

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

    /// CHECK: Edition marker account
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        space = utils::MINT_SPACE,
        owner = token_program.key(),
    )]
    /// CHECK: Initialized as mint in instruction
    pub mint: AccountInfo<'info>,

    /// CHECK: Mint authority PDA
    #[account(
        seeds = ["mint_authority".as_bytes(), mint.key().as_ref()],
        bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: Token account to be initialized
    #[account(mut)]
    pub token_account: AccountInfo<'info>,

    /// CHECK: Collection authority record from master state
    #[account(
        mut,
        address = master_state.collection_authority_record
    )]
    pub collection_authority_record: UncheckedAccount<'info>,

    /// CHECK: Delegate authority from master state  
    #[account(
        mut,
        seeds = ["collection_delegate".as_bytes(), master_mint.key().as_ref()],
        bump,  
    )]
    pub delegate_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    /// CHECK: Required by token metadata program
    #[account(address = METADATA_PROGRAM_ID @ CustomError::InvalidProgramId)]
    pub token_metadata_program: UncheckedAccount<'info>,
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
        seeds = [
            b"metadata",
            METADATA_PROGRAM_ID.as_ref(),
            mint.key().as_ref(),
        ],
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
        seeds = [
            b"metadata",
            METADATA_PROGRAM_ID.as_ref(),
            mint.key().as_ref(),
        ],
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
        seeds = [
            b"metadata", 
            METADATA_PROGRAM_ID.as_ref(),
            mint.key().as_ref(),
            b"edition",
            token_program.key().as_ref()
        ],
        bump,
        seeds::program = METADATA_PROGRAM_ID
    )]
    pub edition_marker: UncheckedAccount<'info>,
}

#[account]
pub struct MasterState {
    pub master_mint: Pubkey,
    pub total_minted: u64,
    pub collection_delegate: Pubkey,
    pub collection_authority_record: Pubkey,
}

impl MasterState {
    pub const SPACE: usize = 8 + 32 + 8 + 32 + 32; // discriminator + master_mint + total_minted + delegate + authority_record
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

        // Initialize master mint
        token::initialize_mint(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::InitializeMint {
                    mint: ctx.accounts.master_mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            0,
            &ctx.accounts.update_authority.key(),
            Some(&ctx.accounts.update_authority.key()),
        )?;

        // Create update authority's ATA
        let cpi_accounts = Create {
            payer: ctx.accounts.payer.to_account_info(),
            associated_token: ctx.accounts.update_authority_token.to_account_info(),
            authority: ctx.accounts.update_authority.to_account_info(),
            mint: ctx.accounts.master_mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };

        let cpi_context = CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            cpi_accounts,
        );

        associated_token::create(cpi_context)?;

        // Mint one token to update authority's ATA
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.master_mint.to_account_info(),
                    to: ctx.accounts.update_authority_token.to_account_info(),
                    authority: ctx.accounts.update_authority.to_account_info(),
                },
            ),
            1,
        )?;

        // Create metadata
        let metadata_data = DataV2 {
            name: NAME.to_string(),
            symbol: SYMBOL.to_string(),
            uri: URI.to_string(),
            seller_fee_basis_points: SELLER_FEE_BASIS_POINTS,
            creators: None,
            collection: None,
            uses: None,
        };

        let create_metadata_ix = CreateMetadataAccountV3Builder::new()
            .metadata(ctx.accounts.master_metadata.key())
            .mint(ctx.accounts.master_mint.key())
            .mint_authority(ctx.accounts.update_authority.key())
            .payer(ctx.accounts.payer.key())
            .update_authority(ctx.accounts.update_authority.key(), true)
            .data(metadata_data)
            .is_mutable(true)
            .instruction();

        invoke_signed(
            &create_metadata_ix,
            &[
                ctx.accounts.master_metadata.to_account_info(),
                ctx.accounts.master_mint.to_account_info(),
                ctx.accounts.update_authority.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.token_metadata_program.to_account_info(),
            ],
            &[],
        )?;

        // Create master edition
        let create_master_edition_ix = CreateMasterEditionV3Builder::new()
            .edition(ctx.accounts.master_edition.key())
            .mint(ctx.accounts.master_mint.key())
            .update_authority(ctx.accounts.update_authority.key())
            .mint_authority(ctx.accounts.update_authority.key())
            .metadata(ctx.accounts.master_metadata.key())
            .payer(ctx.accounts.payer.key())
            .max_supply(0)
            .instruction();

        invoke_signed(
            &create_master_edition_ix,
            &[
                ctx.accounts.master_edition.to_account_info(),
                ctx.accounts.master_mint.to_account_info(),
                ctx.accounts.update_authority.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.master_metadata.to_account_info(),
                ctx.accounts.token_metadata_program.to_account_info(),
            ],
            &[],
        )?;

        let set_and_verify_collection_ix =
            mpl_token_metadata::instructions::SetAndVerifyCollectionBuilder::new()
                .metadata(ctx.accounts.master_metadata.key())
                .collection_authority(ctx.accounts.update_authority.key())
                .update_authority(ctx.accounts.update_authority.key())
                .payer(ctx.accounts.payer.key())
                .collection_mint(ctx.accounts.master_mint.key())
                .collection(ctx.accounts.master_metadata.key())
                .collection_master_edition_account(ctx.accounts.master_edition.key())
                .collection_authority_record(None)
                .instruction();

        invoke_signed(
            &set_and_verify_collection_ix,
            &[
                ctx.accounts.master_metadata.to_account_info(),
                ctx.accounts.update_authority.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.master_mint.to_account_info(),
                ctx.accounts.master_metadata.to_account_info(),
                ctx.accounts.master_edition.to_account_info(),
                ctx.accounts.token_metadata_program.to_account_info(),
            ],
            &[],
        )?;

        // Setup collection authority delegate
        // Derive the delegate authority PDA
        let (delegate_authority, _) = Pubkey::find_program_address(
            &[
                b"collection_delegate",
                ctx.accounts.master_mint.key().as_ref(),
            ],
            ctx.program_id,
        );
        msg!("Derived delegate authority: {}", delegate_authority);

        msg!(
            "Collection metadata owner: {}",
            ctx.accounts.master_metadata.owner
        );
        msg!(
            "Collection authority record owner: {}",
            ctx.accounts.collection_authority_record.owner
        );

        // Use Metaplex's approve_collection_authority function
        let approve_collection_authority_ix =
            mpl_token_metadata::instructions::ApproveCollectionAuthorityBuilder::new()
                .collection_authority_record(ctx.accounts.collection_authority_record.key())
                .new_collection_authority(delegate_authority)
                .update_authority(ctx.accounts.update_authority.key())
                .payer(ctx.accounts.payer.key())
                .metadata(ctx.accounts.master_metadata.key())
                .mint(ctx.accounts.master_mint.key())
                .instruction();

        invoke(
            &approve_collection_authority_ix,
            &[
                ctx.accounts.master_metadata.to_account_info(),
                ctx.accounts.update_authority.to_account_info(),
                ctx.accounts.collection_authority_record.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.master_mint.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.delegate_authority.to_account_info(),
            ],
        )?;

        msg!("State after approve_collection_authority:");
        msg!(
            "Collection authority record: {}",
            ctx.accounts.collection_authority_record.key()
        );
        msg!(
            "Collection metadata: {}",
            ctx.accounts.master_metadata.key()
        );
        msg!("Delegate authority: {}", delegate_authority);

        master_state.collection_delegate = delegate_authority;
        master_state.collection_authority_record = ctx.accounts.collection_authority_record.key();

        Ok(())
    }

    pub fn mint_pnft(ctx: Context<MintPNFT>) -> Result<()> {
        msg!("Starting mint_pnft instruction");
        msg!("Payer: {}", ctx.accounts.payer.key());
        msg!("Vault: {}", ctx.accounts.vault.key());
        msg!("Master State: {}", ctx.accounts.master_state.key());
        msg!(
            "Collection Metadata: {}",
            ctx.accounts.collection_metadata.key()
        );
        msg!(
            "Collection Master Edition: {}",
            ctx.accounts.collection_master_edition.key()
        );
        msg!("Metadata: {}", ctx.accounts.metadata.key());
        msg!("Master Edition: {}", ctx.accounts.master_edition.key());
        msg!("Mint: {}", ctx.accounts.mint.key());
        msg!("Mint Authority: {}", ctx.accounts.mint_authority.key());
        msg!("Token Account: {}", ctx.accounts.token_account.key());
        msg!("Token Program: {}", ctx.accounts.token_program.key());
        msg!(
            "Associated Token Program: {}",
            ctx.accounts.associated_token_program.key()
        );
        msg!("System Program: {}", ctx.accounts.system_program.key());
        msg!("Rent: {}", ctx.accounts.rent.key());
        msg!(
            "Token Metadata Program: {}",
            ctx.accounts.token_metadata_program.key()
        );

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

        // Create metadata (with mint_authority as update authority)
        create_metadata(
            &ctx,
            get_initial_metadata(collection),
            &[&mint_authority_seeds],
        )?;

        msg!(
            "Collection metadata owner: {}",
            ctx.accounts.collection_metadata.owner
        );
        msg!(
            "Collection authority record owner: {}",
            ctx.accounts.collection_authority_record.owner
        );
        msg!("Metadata account owner: {}", ctx.accounts.metadata.owner);

        // Verify collection
        verify_collection(&ctx)?;

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
    ctx: &Context<'_, '_, '_, 'info, MintPNFT>,
    metadata_data: DataV2,
    signing_seeds: &[&[&[u8]]],
) -> Result<()> {
    let create_metadata_ix = CreateMetadataAccountV3Builder::new()
        .metadata(ctx.accounts.metadata.key())
        .mint(ctx.accounts.mint.key())
        .mint_authority(ctx.accounts.mint_authority.key())
        .payer(ctx.accounts.payer.key())
        .update_authority(ctx.accounts.mint_authority.key(), true)
        .data(metadata_data)
        .is_mutable(true)
        .instruction();

    invoke_signed(
        &create_metadata_ix,
        &[
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
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

fn verify_collection<'info>(ctx: &Context<'_, '_, '_, 'info, MintPNFT>) -> Result<()> {
    let master_mint_key = ctx.accounts.master_mint.key();
    let delegate_seeds = &[
        b"collection_delegate",
        master_mint_key.as_ref(),
        &[ctx.bumps.delegate_authority],
    ];

    msg!(
        "Verifying collection with delegate authority: {}",
        ctx.accounts.delegate_authority.key()
    );
    msg!("Using delegate bump: {}", ctx.bumps.delegate_authority);

    let verify_collection_ix = VerifyCollectionBuilder::new()
        .metadata(ctx.accounts.metadata.key())
        .collection_authority(ctx.accounts.delegate_authority.key())
        .payer(ctx.accounts.payer.key())
        .collection_mint(ctx.accounts.master_mint.key())
        .collection(ctx.accounts.collection_metadata.key())
        .collection_master_edition_account(ctx.accounts.collection_master_edition.key())
        .collection_authority_record(Some(ctx.accounts.collection_authority_record.key()))
        .instruction();

    invoke_signed(
        &verify_collection_ix,
        &[
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.master_mint.to_account_info(),
            ctx.accounts.delegate_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.collection_metadata.to_account_info(),
            ctx.accounts.collection_master_edition.to_account_info(),
            ctx.accounts.collection_authority_record.to_account_info(),
        ],
        &[delegate_seeds],
    )?;

    Ok(())
}

fn burn_nft(ctx: &Context<BurnAndWithdraw>) -> Result<()> {
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

    let burn_ix = BurnBuilder::new()
        .authority(ctx.accounts.owner.key())
        .metadata(ctx.accounts.metadata.key())
        .mint(ctx.accounts.mint.key())
        .token(ctx.accounts.token_account.key())
        .edition_marker(Some(ctx.accounts.edition_marker.key()))
        .instruction();

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
