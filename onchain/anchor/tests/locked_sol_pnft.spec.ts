import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { SendTransactionError } from '@solana/web3.js'
import { LockedSolPnft } from '../target/types/locked_sol_pnft'
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  Keypair,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js'
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from '@solana/spl-token'
import { Metadata, PROGRAM_ID as TOKEN_METADATA_PROGRAM_ID } from '@metaplex-foundation/mpl-token-metadata'
import * as fs from 'fs'
import * as path from 'path'

const getMasterStateAddress = async (masterMint: PublicKey, program: Program<LockedSolPnft>): Promise<PublicKey> => {
  return (await PublicKey.findProgramAddress([Buffer.from('master'), masterMint.toBuffer()], program.programId))[0]
}

const getMasterMintAddress = async (program: Program<LockedSolPnft>): Promise<PublicKey> => {
  return (await PublicKey.findProgramAddress([Buffer.from('master_mint')], program.programId))[0]
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
      [Buffer.from('metadata'), mint.toBuffer(), TOKEN_PROGRAM_ID.toBuffer()],
      TOKEN_METADATA_PROGRAM_ID,
    )
  )[0]
}

const getMasterMetadataAddress = async (masterMint: PublicKey): Promise<PublicKey> => {
  return (
    await PublicKey.findProgramAddress(
      [Buffer.from('metadata'), masterMint.toBuffer(), TOKEN_PROGRAM_ID.toBuffer()],
      TOKEN_METADATA_PROGRAM_ID,
    )
  )[0]
}

const getMasterEditionAddress = async (mint: PublicKey): Promise<PublicKey> => {
  return (
    await PublicKey.findProgramAddress(
      [Buffer.from('metadata'), mint.toBuffer(), Buffer.from('edition')],
      TOKEN_METADATA_PROGRAM_ID,
    )
  )[0]
}

const getEditionMarkerAddress = async (mint: PublicKey): Promise<PublicKey> => {
  return (
    await PublicKey.findProgramAddress(
      [Buffer.from('metadata'), mint.toBuffer(), Buffer.from('edition')],
      TOKEN_METADATA_PROGRAM_ID,
    )
  )[0]
}

describe('locked-sol-pnft', () => {
  const serverAuthorityKeypair = Keypair.fromSecretKey(
    Uint8Array.from([
      74, 235, 218, 204, 165, 249, 251, 252, 37, 204, 109, 249, 38, 87, 204, 248, 146, 22, 17, 96, 195, 210, 85, 28,
      153, 179, 176, 0, 17, 63, 130, 49, 203, 190, 104, 203, 128, 153, 190, 207, 232, 27, 224, 77, 215, 94, 23, 146, 12,
      111, 140, 15, 14, 222, 232, 215, 149, 202, 162, 75, 17, 180, 152, 182,
    ]),
  )

  console.log('Server Authority Pubkey:', serverAuthorityKeypair.publicKey.toBase58())

  const UPDATE_AUTHORITY_PUBKEY = serverAuthorityKeypair.publicKey

  const walletKeypair = Keypair.fromSecretKey(
    Buffer.from(JSON.parse(fs.readFileSync(path.join(__dirname, '../../../keys/devnet.json'), 'utf-8'))),
  )

  const provider = new anchor.AnchorProvider(anchor.getProvider().connection, new anchor.Wallet(walletKeypair), {})
  anchor.setProvider(provider)

  const program = anchor.workspace.LockedSolPnft as Program<LockedSolPnft>

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
    try {
      // Fund the test update authority
      const balance = await provider.connection.getBalance(UPDATE_AUTHORITY_PUBKEY)
      console.log('Current server authority balance:', balance / LAMPORTS_PER_SOL, 'SOL')

      if (balance < LAMPORTS_PER_SOL) {
        console.log('Funding server authority account...')
        const signature = await provider.connection.requestAirdrop(UPDATE_AUTHORITY_PUBKEY, 2 * LAMPORTS_PER_SOL)

        const latestBlockHash = await provider.connection.getLatestBlockhash()
        await provider.connection.confirmTransaction({
          signature,
          blockhash: latestBlockHash.blockhash,
          lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
        })

        const newBalance = await provider.connection.getBalance(UPDATE_AUTHORITY_PUBKEY)
        console.log('New server authority balance:', newBalance / LAMPORTS_PER_SOL, 'SOL')

        if (newBalance < LAMPORTS_PER_SOL) {
          throw new Error('Failed to fund server authority account')
        }
      }

      // Get master mint and related addresses
      masterMintPubkey = await getMasterMintAddress(program)
      console.log('Master Mint Address:', masterMintPubkey.toBase58())

      const [masterState, masterMetadata, masterEdition] = await Promise.all([
        getMasterStateAddress(masterMintPubkey, program),
        getMasterMetadataAddress(masterMintPubkey),
        getMasterEditionAddress(masterMintPubkey),
      ])

      masterStatePubkey = masterState
      masterMetadataPubkey = masterMetadata
      masterEditionPubkey = masterEdition

      // Check if master edition exists
      const masterEditionAccount = await provider.connection.getAccountInfo(masterEditionPubkey)

      if (!masterEditionAccount) {
        console.log('Initializing master edition...')
        // Initialize master edition if it doesn't exist
        await program.methods
          .initializeMasterEdition()
          .accountsStrict({
            payer: provider.wallet.publicKey,
            masterState: masterStatePubkey,
            masterMint: masterMintPubkey,
            masterMetadata: masterMetadataPubkey,
            masterEdition: masterEditionPubkey,
            updateAuthority: UPDATE_AUTHORITY_PUBKEY,
            tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([walletKeypair])
          .rpc()

        // Verify initialization
        const masterStateAccount = await program.account.masterState.fetch(masterStatePubkey)
        console.log('Master state initialized:', {
          masterMint: masterStateAccount.masterMint.toBase58(),
          totalMinted: masterStateAccount.totalMinted.toString(),
        })
      } else {
        console.log('Master edition already exists')
        // Verify existing state
        const masterStateAccount = await program.account.masterState.fetch(masterStatePubkey)
        console.log('Existing master state:', {
          masterMint: masterStateAccount.masterMint.toBase58(),
          totalMinted: masterStateAccount.totalMinted.toString(),
        })
      }

      // Verify the state - move this inside the try block
      const masterStateAccount = await program.account.masterState.fetch(masterStatePubkey)
      expect(masterStateAccount.masterMint).toEqual(masterMintPubkey)
      expect(masterStateAccount.totalMinted.toString()).toBe('0')
    } catch (error) {
      console.error('Error during setup:', error)
      if (error instanceof SendTransactionError) {
        console.log('Transaction logs:', error.logs)
      }
      throw error
    }
  })

  beforeEach(async () => {
    mint = Keypair.generate()

    const [vault, mintAuthority, metadata, editionMarker] = await Promise.all([
      getVaultAddress(mint.publicKey, program),
      getMintAuthorityAddress(mint.publicKey, program),
      getMetadataAddress(mint.publicKey),
      getEditionMarkerAddress(mint.publicKey),
    ])

    vaultPubkey = vault
    mintAuthorityPubkey = mintAuthority
    metadataPubkey = metadata
    editionPubkey = editionMarker
    tokenAccountPubkey = await getAssociatedTokenAddress(mint.publicKey, provider.wallet.publicKey)

    await program.methods
      .mintPnft()
      .accountsStrict({
        payer: provider.wallet.publicKey,
        vault: vaultPubkey,
        masterState: masterStatePubkey,
        collectionMetadata: masterMetadataPubkey,
        collectionMasterEdition: masterEditionPubkey,
        metadata: metadataPubkey,
        editionMarker: editionPubkey,
        mint: mint.publicKey,
        mintAuthority: mintAuthorityPubkey,
        serverAuthority: UPDATE_AUTHORITY_PUBKEY,
        tokenAccount: tokenAccountPubkey,
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

  it('Should update metadata', async () => {
    const newUri = 'https://api.locked-sol.com/metadata/updated.json'
    const newName = 'Updated NFT Name'

    await program.methods
      .updateMetadata(newUri, newName)
      .accountsStrict({
        serverAuthority: UPDATE_AUTHORITY_PUBKEY,
        vault: vaultPubkey,
        masterState: masterStatePubkey,
        metadata: metadataPubkey,
        mint: mint.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([serverAuthorityKeypair])
      .rpc()

    const metadataAccount = await Metadata.fromAccountAddress(provider.connection, metadataPubkey)
    expect(metadataAccount.data.uri).toBe(newUri)
    expect(metadataAccount.data.name).toBe(newName)
  })

  it('Should burn and withdraw', async () => {
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
