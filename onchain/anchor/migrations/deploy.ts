// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

import * as anchor from '@coral-xyz/anchor'
import { AnchorProvider } from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { LockedSolPnft } from '../target/types/locked_sol_pnft'
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from '@solana/web3.js'
import { TOKEN_PROGRAM_ID } from '@solana/spl-token'

module.exports = async function (provider: anchor.Provider) {
  // Configure client
  anchor.setProvider(provider)
  const program = anchor.workspace.LockedSolPnft as Program<LockedSolPnft>

  const TOKEN_METADATA_PROGRAM_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s')

  // Derive necessary PDAs
  const [masterMint] = PublicKey.findProgramAddressSync([Buffer.from('master_mint')], program.programId)

  const [masterState] = PublicKey.findProgramAddressSync(
    [Buffer.from('master'), masterMint.toBuffer()],
    program.programId,
  )

  const [masterMetadata] = PublicKey.findProgramAddressSync(
    [Buffer.from('metadata'), TOKEN_METADATA_PROGRAM_ID.toBuffer(), masterMint.toBuffer()],
    TOKEN_METADATA_PROGRAM_ID,
  )

  const [masterEdition] = PublicKey.findProgramAddressSync(
    [Buffer.from('metadata'), TOKEN_METADATA_PROGRAM_ID.toBuffer(), masterMint.toBuffer(), Buffer.from('edition')],
    TOKEN_METADATA_PROGRAM_ID,
  )

  console.log('Initializing master edition...')
  console.log('Program ID:', program.programId.toBase58())
  console.log('Master Mint:', masterMint.toBase58())
  console.log('Master State:', masterState.toBase58())
  console.log('Master Metadata:', masterMetadata.toBase58())
  console.log('Master Edition:', masterEdition.toBase58())

  const anchorProvider = provider as AnchorProvider
  try {
    const tx = await program.methods
      .initializeMasterEdition()
      .accounts({
        payer: anchorProvider.wallet.publicKey,
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

    console.log('Master edition initialized successfully!')
    console.log('Transaction signature:', tx)

    // Verify initialization
    const masterStateAccount = await program.account.masterState.fetch(masterState)
    console.log('Master state initialized with:')
    console.log('- Master Mint:', masterStateAccount.masterMint.toBase58())
    console.log('- Total Minted:', masterStateAccount.totalMinted.toString())
  } catch (error) {
    console.error('Error initializing master edition:', error)
    throw error
  }
}
