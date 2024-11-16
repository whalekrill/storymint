import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { LockedSolPnft } from '../target/types/locked_sol_pnft'
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  Keypair,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js'
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  MintLayout,
  createInitializeMintInstruction,
} from '@solana/spl-token'
import { Metadata, PROGRAM_ID as TOKEN_METADATA_PROGRAM_ID } from '@metaplex-foundation/mpl-token-metadata'

const getMasterStateAddress = async (masterMint: PublicKey, program: Program<LockedSolPnft>): Promise<PublicKey> => {
  return (await PublicKey.findProgramAddress([Buffer.from('master'), masterMint.toBuffer()], program.programId))[0]
}

const getMasterMintAddress = async (program: Program<LockedSolPnft>): Promise<PublicKey> => {
  return (await PublicKey.findProgramAddress([Buffer.from('master_mint')], program.programId))[0]
}

const getMasterEditionAddress = async (mint: PublicKey): Promise<PublicKey> => {
  return (
    await PublicKey.findProgramAddress(
      [Buffer.from('metadata'), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer(), Buffer.from('edition')],
      TOKEN_METADATA_PROGRAM_ID,
    )
  )[0]
}

const getVaultAddress = async (mint: PublicKey, program: Program<LockedSolPnft>): Promise<PublicKey> => {
  return (await PublicKey.findProgramAddress([Buffer.from('vault'), mint.toBuffer()], program.programId))[0]
}

const getMintAuthorityAddress = async (mint: PublicKey, program: Program<LockedSolPnft>): Promise<PublicKey> => {
  return (await PublicKey.findProgramAddress([Buffer.from('mint_authority'), mint.toBuffer()], program.programId))[0]
}

const getMetadataAddress = async (mint: PublicKey): Promise<PublicKey> => {
  return (
    await PublicKey.findProgramAddress(
      [Buffer.from('metadata'), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      TOKEN_METADATA_PROGRAM_ID,
    )
  )[0]
}

const getEditionMarkerAddress = async (mint: PublicKey): Promise<PublicKey> => {
  return (
    await PublicKey.findProgramAddress(
      [Buffer.from('metadata'), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer(), Buffer.from('edition')],
      TOKEN_METADATA_PROGRAM_ID,
    )
  )[0]
}

describe('locked-sol-pnft', () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace.LockedSolPnft as Program<LockedSolPnft>

  const SERVER_UPDATE_AUTHORITY = new PublicKey('DJ4xnt8cNHFXehsHFQqyNB2KjXHjYyYBX5565wKAhRaR')
  const VAULT_AMOUNT = 1 * LAMPORTS_PER_SOL

  let masterMintPubkey: PublicKey
  let masterStatePubkey: PublicKey
  let masterMetadataPubkey: PublicKey
  let masterEditionPubkey: PublicKey

  let mint: Keypair
  let vaultPubkey: PublicKey
  let metadataPubkey: PublicKey
  let editionPubkey: PublicKey
  let tokenAccountPubkey: PublicKey
  let mintAuthorityPubkey: PublicKey

  beforeAll(async () => {
    const masterMint = await getMasterMintAddress(program)

    // Then derive the rest of the addresses
    const [masterState, masterMetadata, masterEdition] = await Promise.all([
      getMasterStateAddress(masterMint, program),
      getMetadataAddress(masterMint),
      getMasterEditionAddress(masterMint),
    ])

    // Submit transaction with properly named accounts matching IDL
    const tx = await program.methods
      .initializeMasterEdition()
      .accountsStrict({
        payer: provider.wallet.publicKey,
        masterState,
        masterMint,
        masterMetadata,
        masterEdition,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .rpc()

    console.log('Transaction signature:', tx)

    const masterStateAccount = await program.account.masterState.fetch(masterStatePubkey)
    expect(masterStateAccount.masterMint).toEqual(masterMintPubkey)
    expect(masterStateAccount.totalMinted.toString()).toBe('0')
  })

  beforeEach(async () => {
    const masterMint = await getMasterMintAddress(program)

    // Then derive the rest of the addresses
    const masterState = await getMasterStateAddress(masterMint, program)

    mint = Keypair.generate()

    const [vault, mintAuthority, metadata, editionMarker] = await Promise.all([
      getVaultAddress(mint.publicKey, program),
      getMintAuthorityAddress(mint.publicKey, program),
      getMetadataAddress(mint.publicKey),
      getEditionMarkerAddress(mint.publicKey),
    ])

    const tokenAccount = await getAssociatedTokenAddress(mint.publicKey, provider.wallet.publicKey)

    // Initialize mint
    const lamports = await provider.connection.getMinimumBalanceForRentExemption(MintLayout.span)
    const createMintTx = new anchor.web3.Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: provider.wallet.publicKey,
        newAccountPubkey: mint.publicKey,
        space: MintLayout.span,
        lamports,
        programId: TOKEN_PROGRAM_ID,
      }),
      createInitializeMintInstruction(
        mint.publicKey,
        0,
        provider.wallet.publicKey,
        provider.wallet.publicKey,
        TOKEN_PROGRAM_ID,
      ),
    )

    await provider.sendAndConfirm(createMintTx, [mint])

    await program.methods
      .mintPnft()
      .accountsStrict({
        payer: provider.wallet.publicKey,
        vault,
        masterState,
        collectionMetadata: masterMetadataPubkey,
        collectionMasterEdition: masterEditionPubkey,
        metadata,
        editionMarker,
        mint: mint.publicKey,
        mintAuthority,
        serverAuthority: SERVER_UPDATE_AUTHORITY,
        tokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([mint])
      .rpc()

    const vaultAccount = await program.account.tokenVault.fetch(vaultPubkey)
    expect(vaultAccount.mint).toEqual(mint.publicKey)

    const tokenBalance = await provider.connection.getTokenAccountBalance(tokenAccountPubkey)
    expect(tokenBalance.value.uiAmount).toBe(1)
  })

  test('Update Metadata', async () => {
    const newUri = 'https://api.locked-sol.com/metadata/updated.json'
    const newName = 'Updated NFT Name'

    await program.methods
      .updateMetadata(newUri, newName)
      .accountsStrict({
        serverAuthority: SERVER_UPDATE_AUTHORITY,
        vault: vaultPubkey,
        masterState: masterStatePubkey,
        metadata: metadataPubkey,
        mint: mint.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc()

    const metadataAccount = await Metadata.fromAccountAddress(provider.connection, metadataPubkey)
    expect(metadataAccount.data.uri).toBe(newUri)
    expect(metadataAccount.data.name).toBe(newName)
  })

  test('Burn and Withdraw', async () => {
    const initialBalance = await provider.connection.getBalance(provider.wallet.publicKey)

    await program.methods
      .burnAndWithdraw()
      .accountsStrict({
        owner: provider.wallet.publicKey,
        masterState: masterStatePubkey,
        vault: vaultPubkey,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        metadata: metadataPubkey,
        tokenAccount: tokenAccountPubkey,
        mint: mint.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
        sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
        editionMarker: editionPubkey,
      })
      .rpc()

    const finalBalance = await provider.connection.getBalance(provider.wallet.publicKey)
    expect(finalBalance).toBeGreaterThan(initialBalance)
  })
})
