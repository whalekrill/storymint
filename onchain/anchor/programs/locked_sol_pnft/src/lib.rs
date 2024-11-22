use anchor_lang::prelude::*;
use anchor_lang::system_program;
use mpl_core::types::{DataState, PluginAuthorityPair};

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
pub struct InitializeCollection<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = MasterState::SPACE,
        seeds = ["master".as_bytes(), collection.key().as_ref()],
        bump
    )]
    pub master_state: Account<'info, MasterState>,

    /// CHECK: Initialized by MPL Core
    #[account(mut)]
    pub collection: AccountInfo<'info>,

    #[account(
        mut,
        signer,
        constraint = update_authority.key() == SERVER_AUTHORITY @ CustomError::InvalidUpdateAuthority
    )]
    pub update_authority: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    /// CHECK: MPL Core program
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct MintAsset<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = TokenVault::SPACE,
        seeds = ["vault".as_bytes(), asset.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, TokenVault>,

    /// The new asset being created
    #[account(mut)]
    pub asset: Signer<'info>,

    #[account(
        mut,
        seeds = ["master".as_bytes(), collection.key().as_ref()],
        bump,
        constraint = master_state.total_minted < MAX_SUPPLY @ CustomError::MaxSupplyReached,
        has_one = collection @ CustomError::InvalidCollection
    )]
    pub master_state: Account<'info, MasterState>,

    /// The collection this asset belongs to
    /// CHECK: Checked in mpl-core
    #[account(mut)]
    pub collection: AccountInfo<'info>,

    /// The authority signing for creation (optional)
    pub authority: Option<Signer<'info>>,

    /// The owner of the new asset
    /// CHECK: Checked in mpl-core
    pub owner: Option<AccountInfo<'info>>,

    /// The authority on the new asset
    /// CHECK: Checked in mpl-core
    pub update_authority: Option<AccountInfo<'info>>,

    pub system_program: Program<'info, System>,

    /// CHECK: SPL Noop program
    pub log_wrapper: Option<AccountInfo<'info>>,

    /// CHECK: MPL Core program
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct MintAssetArgs {
    pub name: String,
    pub uri: String,
    pub plugins: Option<Vec<PluginAuthorityPair>>,
}

#[derive(Accounts)]
pub struct UpdateMetadata<'info> {
    /// The asset to update
    /// CHECK: Checked in mpl-core
    #[account(mut)]
    pub asset: AccountInfo<'info>,

    /// The collection this asset belongs to (optional)
    /// CHECK: Checked in mpl-core
    #[account(mut)]
    pub collection: Option<AccountInfo<'info>>,

    #[account(
        mut, 
        signer,
        constraint = authority.key() == SERVER_AUTHORITY @ CustomError::UnauthorizedUpdate
    )]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: SPL Noop program
    pub log_wrapper: Option<AccountInfo<'info>>,

    /// CHECK: MPL Core program
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct UpdateMetadataArgs {
    pub name: Option<String>,
    pub uri: Option<String>,
}

#[derive(Accounts)]
pub struct BurnAndWithdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    /// CHECK: Asset being burned, checked by MPL Core
    #[account(mut)]
    pub asset: AccountInfo<'info>,

    /// CHECK: Collection the asset belongs to
    #[account(mut)]
    pub collection: AccountInfo<'info>,

    #[account(
        mut,
        seeds = ["master".as_bytes(), collection.key().as_ref()],
        bump
    )]
    pub master_state: Account<'info, MasterState>,

    #[account(
        mut,
        seeds = ["vault".as_bytes(), asset.key().as_ref()],
        bump,
        close = owner
    )]
    pub vault: Account<'info, TokenVault>,

    pub system_program: Program<'info, System>,

    /// CHECK: SPL Noop program
    pub log_wrapper: Option<AccountInfo<'info>>,

    /// CHECK: MPL Core program
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,
}

#[account]
pub struct MasterState {
    pub collection: Pubkey,
    pub total_minted: u64,
}

impl MasterState {
    pub const SPACE: usize = 8 + 32 + 8; // discriminator + collection + total_minted
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
    #[msg("Maximum supply reached")]
    MaxSupplyReached,
    #[msg("Invalid collection data")]
    InvalidCollection,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Arithmetic underflow")]
    Underflow,
    #[msg("Invalid update authority")]
    InvalidUpdateAuthority,
}

#[program]
pub mod locked_sol_pnft {
    use super::*;

    pub fn initialize_collection(
        ctx: Context<InitializeCollection>,
        name: String,
        uri: String,
    ) -> Result<()> {
        let master_state = &mut ctx.accounts.master_state;
        master_state.total_minted = 0;

        mpl_core::instructions::CreateCollectionV1Cpi {
            collection: ctx.accounts.collection.as_ref(),
            payer: &ctx.accounts.payer.to_account_info(),
            update_authority: Some(ctx.accounts.update_authority.as_ref()),
            system_program: &ctx.accounts.system_program.to_account_info(),
            __program: &ctx.accounts.mpl_core,
            __args: mpl_core::instructions::CreateCollectionV1InstructionArgs {
                name,
                uri,
                plugins: None,
            },
        }
        .invoke()?;

        Ok(())
    }

    pub fn mint_asset(ctx: Context<MintAsset>, args: MintAssetArgs) -> Result<()> {
        let rent = Rent::get()?;
        let rent_costs = utils::calculate_rent(&rent, true);
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

        ctx.accounts.vault.mint = ctx.accounts.asset.key();

        // Create the asset using MPL Core
        mpl_core::instructions::CreateV1Cpi {
            asset: &ctx.accounts.asset.to_account_info(),
            collection: Some(ctx.accounts.collection.as_ref()),
            authority: ctx.accounts.authority.as_deref(),
            payer: &ctx.accounts.payer.to_account_info(),
            owner: ctx.accounts.owner.as_ref(),
            update_authority: ctx.accounts.update_authority.as_ref(),
            system_program: &ctx.accounts.system_program.to_account_info(),
            log_wrapper: ctx.accounts.log_wrapper.as_ref(),
            __program: &ctx.accounts.mpl_core,
            __args: mpl_core::instructions::CreateV1InstructionArgs {
                data_state: DataState::AccountState,
                name: args.name,
                uri: args.uri,
                plugins: args.plugins,
            },
        }
        .invoke()?;

        ctx.accounts.master_state.total_minted = ctx
            .accounts
            .master_state
            .total_minted
            .checked_add(1)
            .ok_or(CustomError::Overflow)?;

        Ok(())
    }
    pub fn update_metadata(ctx: Context<UpdateMetadata>, args: UpdateMetadataArgs) -> Result<()> {
        mpl_core::instructions::UpdateV1Cpi {
            asset: &ctx.accounts.asset.to_account_info(),
            collection: ctx.accounts.collection.as_ref(),
            authority: Some(ctx.accounts.authority.as_ref()),
            payer: &ctx.accounts.payer.to_account_info(),
            system_program: &ctx.accounts.system_program.to_account_info(),
            log_wrapper: ctx.accounts.log_wrapper.as_ref(),
            __program: &ctx.accounts.mpl_core,
            __args: mpl_core::instructions::UpdateV1InstructionArgs {
                new_name: args.name,
                new_uri: args.uri,
                new_update_authority: None,
            },
        }
        .invoke()?;

        Ok(())
    }

    pub fn burn_and_withdraw(ctx: Context<BurnAndWithdraw>) -> Result<()> {
        // Burn the asset
        mpl_core::instructions::BurnV1Cpi {
            asset: &ctx.accounts.asset.to_account_info(),
            collection: Some(ctx.accounts.collection.as_ref()),
            authority: Some(ctx.accounts.owner.as_ref()),
            payer: &ctx.accounts.owner.to_account_info(),
            system_program: Some(&ctx.accounts.system_program.to_account_info()),
            log_wrapper: ctx.accounts.log_wrapper.as_ref(),
            __program: &ctx.accounts.mpl_core,
            __args: mpl_core::instructions::BurnV1InstructionArgs {
                compression_proof: None,
            },
        }
        .invoke()?;

        // Return SOL from vault
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

        // Update collection stats
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
}
